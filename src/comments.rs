//! Types and functions for post comments.

use std::cmp;

use tui::widgets::TableState;

use crate::{
    community::Community,
    counts::Counts,
    posts::{Creator, Post},
    Result,
};

mod comment;

pub use comment::*;

/// Load comments from a file instead of making a call to an endpoint.
///
/// Avoids pinging an API endpoint, and needlessly overloading a server.
///
/// Let's be nice to our friendly Lemmy instances :)
pub fn load_comments(file_name: &str) -> Result<CommentResponses> {
    use std::io::Read;

    let mut file = std::fs::File::open(file_name)?;
    let mut res = String::new();

    file.read_to_string(&mut res)?;

    serde_json::from_str::<CommentResponses>(res.as_str()).map_err(|err| err.into())
}

/// Represents a response to a [Comment] API request.
#[repr(C)]
#[derive(Clone, Debug, Eq, serde::Deserialize, serde::Serialize)]
pub struct CommentResponse {
    pub comment: Comment,
    pub creator: Creator,
    pub post: Post,
    pub community: Community,
    pub counts: Counts,
    pub creator_banned_from_community: bool,
    pub subscribed: String,
    pub saved: bool,
    pub creator_blocked: bool,
    pub level: Option<usize>,
}

impl CommentResponse {
    /// Creates a new [CommentResponse].
    pub const fn new() -> Self {
        Self {
            comment: Comment::new(),
            creator: Creator::new(),
            post: Post::new(),
            community: Community::new(),
            counts: Counts::new(),
            creator_banned_from_community: false,
            subscribed: String::new(),
            saved: false,
            creator_blocked: false,
            level: None,
        }
    }

    /// Gets the comment level of the [CommentResponse].
    pub fn level(&self) -> usize {
        self.level.unwrap_or_default()
    }

    /// Sets the comment level of the [CommentResponse].
    pub fn set_level(&mut self, level: usize) {
        self.level.replace(level);
    }
}

impl PartialEq for CommentResponse {
    fn eq(&self, rhs: &Self) -> bool {
        let self_ids_len = self.comment.path.split('.').count();
        let rhs_ids_len = rhs.comment.path.split('.').count();
        let min_level = cmp::min(self_ids_len - 1, rhs_ids_len - 1);

        let self_id = self
            .comment
            .path
            .split('.')
            .nth(min_level)
            .map(|i| i.parse::<u64>().unwrap_or(0));
        let rhs_id = rhs
            .comment
            .path
            .split('.')
            .nth(min_level)
            .map(|i| i.parse::<u64>().unwrap_or(0));

        self.level == rhs.level
            && self_id == rhs_id
            && self.comment.published == rhs.comment.published
    }
}

impl PartialOrd for CommentResponse {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        let self_id = self.comment.id;

        let self_pos = self
            .comment
            .path
            .split('.')
            .map(|c| c.parse::<u64>().unwrap_or(0))
            .position(|c| c == self_id)
            .unwrap_or(0);

        let self_root = self
            .comment
            .path
            .split('.')
            .map(|c| c.parse::<u64>().unwrap_or(0))
            .nth(1);

        let rhs_id = rhs.comment.id;

        let rhs_pos = rhs
            .comment
            .path
            .split('.')
            .map(|c| c.parse::<u64>().unwrap_or(0))
            .position(|c| c == rhs_id)
            .unwrap_or(0);

        let rhs_root = rhs
            .comment
            .path
            .split('.')
            .map(|c| c.parse::<u64>().unwrap_or(0))
            .nth(1);

        let level = cmp::min(self_pos.saturating_sub(1), rhs_pos.saturating_sub(1));

        let ancestor_ord = self
            .comment
            .path
            .split('.')
            .skip(2)
            .take(level.saturating_sub(2))
            .map(|c| c.parse::<u64>().unwrap_or(0))
            .zip(
                rhs.comment
                    .path
                    .split('.')
                    .skip(2)
                    .take(level.saturating_sub(2))
                    .map(|c| c.parse::<u64>().unwrap_or(0)),
            )
            .fold(self_root.cmp(&rhs_root), |acc, (s, r)| acc.then(s.cmp(&r)));

        let self_child = self.counts.child_count();
        let rhs_child = rhs.counts.child_count();

        let published = self.comment.published.as_str();
        let rhs_published = rhs.comment.published.as_str();

        Some(
            ancestor_ord
                .then(self_pos.cmp(&rhs_pos))
                .then(self_child.cmp(&rhs_child))
                .then(self_id.cmp(&rhs_id))
                .then(published.cmp(&rhs_published)),
        )
    }
}

impl Ord for CommentResponse {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        self.partial_cmp(rhs).unwrap_or(cmp::Ordering::Equal)
    }
}

impl Default for CommentResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents list of responses to a [Comment] API request.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommentResponses {
    pub comments: Vec<CommentResponse>,
}

impl CommentResponses {
    /// Creates a new [CommentResponses] list.
    pub const fn new(comments: Vec<CommentResponse>) -> Self {
        Self { comments }
    }

    /// Gets the list of [CommentResponse]s.
    pub fn comments(&self) -> &[CommentResponse] {
        self.comments.as_ref()
    }
}

/// Download a response to the [CommentList](crate::endpoint::Endpoint) endpoint.
pub async fn dl_comments(url: &str) -> Result<CommentResponses> {
    use std::str::FromStr;

    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let response = client.get(hyper::Uri::from_str(url)?).await?;

    let body = hyper::body::to_bytes(response.into_body()).await?;

    #[cfg(feature = "debug_endpoints")]
    crate::utils::write_to_file("comments.json", &body)?;

    serde_json::from_slice::<CommentResponses>(&body).map_err(|err| err.into())
}

/// Table of [CommentResponse]s for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct CommentResponseTable {
    /// List of [CommentResponse]s.
    pub items: Vec<CommentResponse>,
    /// Current parent comment ID, zero for root level.
    pub comment_id: u64,
    pub page: u64,
    /// Comment level, zero for root level.
    pub level: usize,
    /// [TableState] for drawing [CommentResponseTable].
    pub state: TableState,
}

impl CommentResponseTable {
    /// Creates a new [CommentResponseTable].
    pub fn new(items: Vec<CommentResponse>) -> Self {
        Self {
            items,
            comment_id: 0,
            page: 1,
            level: 0,
            state: TableState::default(),
        }
    }

    /// Gets the list of [Comment] items.
    pub fn items(&self) -> &[CommentResponse] {
        self.items.as_ref()
    }

    /// Gets the current level in the [CommentResponseTable].
    pub fn level(&self) -> usize {
        self.level
    }

    /// Increments the level in the [CommentResponseTable].
    pub fn next_level(&mut self) -> usize {
        self.level += 1;
        self.level
    }

    /// Decrements the level in the [CommentResponseTable].
    pub fn previous_level(&mut self) -> usize {
        self.level = self.level.saturating_sub(1);
        self.level
    }

    /// Changes to the next comments page.
    pub fn next_page(&mut self) {
        self.page += 1;
    }

    pub fn previous_page(&mut self) {
        if self.page > 1 {
            self.page -= 1;
        }
    }

    /// Gets a reference to the current [TableState] at the current level.
    pub fn state(&self) -> &TableState {
        &self.state
    }

    /// Gets a mutable reference to the current [TableState] at the current level.
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    /// Gets an optional reference to the currently selected [CommentResponse] at the current
    /// level.
    pub fn current(&self) -> Option<&CommentResponse> {
        if let Some(i) = self.state.selected() {
            self.items.get(i)
        } else {
            None
        }
    }

    pub fn go_to_parent(&mut self) {
        self.level = self.level.saturating_sub(1);
        self.comment_id = self.parent_id();
    }

    pub fn parent_id(&self) -> u64 {
        self.items
            .iter()
            .find(|c| c.comment.id() == self.comment_id)
            .map(|c| {
                c.comment
                    .path
                    .split('.')
                    .nth(self.level.saturating_add(1))
                    .unwrap_or("0")
                    .parse::<u64>()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Clears the [TableState] selection for the current level.
    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    /// Updates the [TableState] to select the next item at the current level.
    pub fn next(&mut self) {
        let len = self.items.len();
        let i = self.state.selected().map(|i| (i + 1) % len).unwrap_or(0);
        self.state.select(Some(i));
    }

    /// Updates the [TableState] to select the previous item at the current level.
    pub fn previous(&mut self) {
        let len = self.items.len();
        let last = len.saturating_sub(1);
        let i = self
            .state
            .selected()
            .map(|i| if i == 0 { last } else { i.saturating_sub(1) })
            .unwrap_or(last);
        self.state.select(Some(i));
    }

    /// Sorts comments by ID, and path length.
    ///
    /// This recursively sorts comments:
    ///
    /// - first by parent ID (indicated by `level` parameter)
    /// - grouping child posts under parents
    /// - smaller IDs are considered earlier than larger IDs
    ///   - future releases may require explicity checking comment date-time
    ///
    /// Callers should always start with level `1`, unless a special-case dictates something else.
    ///
    /// Parameters:
    ///
    /// `level`: comment path level for sorting comparison
    pub fn sort_comments(&mut self) {
        self.items.sort();
        /*
        let max_len = self.items
            .iter()
            .map(|l| l.comment.path.split('.').count())
            .max()
            .unwrap_or(0);

        (0..=max_len).for_each(|_| self.items.sort());
        */
    }
}

impl From<Vec<CommentResponse>> for CommentResponseTable {
    fn from(val: Vec<CommentResponse>) -> Self {
        Self::new(val)
    }
}

impl From<CommentResponses> for CommentResponseTable {
    fn from(val: CommentResponses) -> Self {
        Self::new(val.comments)
    }
}

impl AsRef<CommentResponseTable> for CommentResponseTable {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<CommentResponseTable> for CommentResponseTable {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_comments() {
        let comments = vec![
            CommentResponse {
                comment: Comment {
                    path: "0.1510313.1511444.1512165".into(),
                    published: "2023-08-04T19:59:29.982921".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1510313.1511444".into(),
                    published: "2023-08-04T19:29:44.539462".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1510313".into(),
                    published: "2023-08-04T18:45:16.126539".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1402429.1436014.1492422".into(),
                    published: "2023-08-04T06:23:05.577465".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1459810.1461116".into(),
                    published: "2023-08-03T08:59:12.227404".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1458729".into(),
                    published: "2023-08-03T06:27:52.372133".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1451065.1456841.1461024".into(),
                    published: "2023-08-03T08:51:59.051645".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1459810".into(),
                    published: "2023-08-03T07:33:09.562685".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1402429.1436014.1463371".into(),
                    published: "2023-08-03T11:14:25.780911".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let exp_comments = vec![
            CommentResponse {
                comment: Comment {
                    path: "0.1402429.1436014.1463371".into(),
                    published: "2023-08-03T11:14:25.780911".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1402429.1436014.1492422".into(),
                    published: "2023-08-04T06:23:05.577465".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1451065.1456841.1461024".into(),
                    published: "2023-08-03T08:51:59.051645".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1458729".into(),
                    published: "2023-08-03T06:27:52.372133".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1459810".into(),
                    published: "2023-08-03T07:33:09.562685".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1459810.1461116".into(),
                    published: "2023-08-03T08:59:12.227404".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1510313".into(),
                    published: "2023-08-04T18:45:16.126539".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1510313.1511444".into(),
                    published: "2023-08-04T19:29:44.539462".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CommentResponse {
                comment: Comment {
                    path: "0.1510313.1511444.1512165".into(),
                    published: "2023-08-04T19:59:29.982921".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let mut comment_responses = CommentResponseTable::new(comments);
        comment_responses.sort_comments();

        let response_paths: Vec<String> = comment_responses
            .items()
            .iter()
            .map(|c| c.comment.path.clone())
            .collect();

        let exp_paths: Vec<String> = exp_comments
            .iter()
            .map(|c| c.comment.path.clone())
            .collect();

        assert_eq!(response_paths, exp_paths);
    }
}
