//! Types and functions for [Post](crate::posts::Post) comments.

use tui::widgets::ListState;

/// Represents a comment on a [Post](crate::posts::Post).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Comment {
    pub id: u64,
    pub creator_id: u64,
    pub post_id: u64,
    pub content: String,
    pub removed: bool,
    pub published: String,
    pub deleted: bool,
    pub ap_id: String,
    pub local: bool,
    pub path: String,
    pub distinguished: bool,
    pub language_id: u64,
}

impl Comment {
    /// Creates a new [Comment].
    pub const fn new() -> Self {
        Self {
            id: 0,
            creator_id: 0,
            post_id: 0,
            content: String::new(),
            removed: false,
            published: String::new(),
            deleted: false,
            ap_id: String::new(),
            local: false,
            path: String::new(),
            distinguished: false,
            language_id: 0,
        }
    }

    /// Gets the [Comment] ID.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Gets the creator ID for the [Comment].
    pub const fn creator_id(&self) -> u64 {
        self.creator_id
    }

    /// Gets the [Post](crate::posts::Post) ID for the [Comment].
    pub const fn post_id(&self) -> u64 {
        self.post_id
    }

    /// Gets the content of the [Comment].
    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    /// Gets whether the [Comment] is deleted.
    pub const fn deleted(&self) -> bool {
        self.deleted
    }

    /// Gets the AP ID of the [Comment].
    pub fn ap_id(&self) -> &str {
        self.ap_id.as_str()
    }

    /// Gets whether the [Comment] is local.
    pub const fn local(&self) -> bool {
        self.local
    }

    /// Gets the path of the [Comment].
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Gets whether the [Comment] is distinguished.
    pub const fn distinguished(&self) -> bool {
        self.distinguished
    }

    /// Gets the language ID for the [Comment].
    pub const fn language_id(&self) -> u64 {
        self.language_id
    }
}

/// List of [Comment]s for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct CommentList {
    pub items: Vec<Comment>,
    pub state: ListState,
}

impl CommentList {
    /// Creates a new [CommentList].
    pub fn new(items: Vec<Comment>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    /// Gets the list of [Comment] items.
    pub fn items(&self) -> &[Comment] {
        self.items.as_ref()
    }

    /// Gets a reference to the current [ListState].
    pub fn state(&self) -> &ListState {
        &self.state
    }

    /// Gets a mutable reference to the current [ListState].
    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    /// Gets an optional reference to the currently selected [Comment].
    pub fn current(&self) -> Option<&Comment> {
        match self.state.selected() {
            Some(i) => Some(&self.items[i]),
            None => None,
        }
    }

    /// Clears the [ListState] selection.
    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    /// Updates the [ListState] to select the next item.
    pub fn next(&mut self) {
        let len = self.items.len();
        let i = match self.state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Updates the [ListState] to select the previous item.
    pub fn previous(&mut self) {
        let len = self.items.len();
        let last = len - 1;
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    last
                } else {
                    i.saturating_sub(1)
                }
            }
            None => last,
        };
        self.state.select(Some(i));
    }
}

impl From<Vec<Comment>> for CommentList {
    fn from(val: Vec<Comment>) -> Self {
        Self::new(val)
    }
}

impl AsRef<CommentList> for CommentList {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<CommentList> for CommentList {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
