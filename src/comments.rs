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

/// Represents list of responses to a [Comment] API request.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommentResponses {
    pub comments: Vec<CommentResponse>,
}

impl CommentResponses {
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
