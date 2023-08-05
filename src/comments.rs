//! Types and functions for post comments.

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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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
        }
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
    pub fn sort_comments(&mut self, level: usize) {
        let max_len = self
            .items
            .iter()
            .map(|l| l.comment.path.split('.').count())
            .max()
            .unwrap_or(0);

        if level < max_len {
            self.items.sort_by(|cr, cs| {
                let cr_ids: Vec<u64> = cr
                    .comment
                    .path
                    .split('.')
                    .map(|p| p.parse::<u64>().unwrap_or(0))
                    .collect();

                let cs_ids: Vec<u64> = cs
                    .comment
                    .path
                    .split('.')
                    .map(|p| p.parse::<u64>().unwrap_or(0))
                    .collect();

                if level < cr_ids.len() && level < cs_ids.len() {
                    let cr_id = &cr_ids[level];
                    let cs_id = &cs_ids[level];

                    cr_id.cmp(cs_id).then(cr_ids.len().cmp(&cs_ids.len()))
                } else {
                    let min_level = std::cmp::min(cr_ids.len() - 1, cs_ids.len() - 1);
                    cr_ids[min_level].cmp(&cs_ids[min_level])
                }
            });

            self.sort_comments(level + 1);
        }
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
                    path: "0.1396433.1402606.1437059.1463746".into(),
                    published: "2023-08-03T11:34:11.084929".into(),
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
                    path: "0.1396433.1402606.1437059.1463746".into(),
                    published: "2023-08-03T11:34:11.084929".into(),
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
        comment_responses.sort_comments(1);

        assert_eq!(comment_responses.items(), exp_comments.as_slice());
    }
}
