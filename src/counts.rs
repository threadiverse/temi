//! Types for representing count statistics.

use tui::widgets::ListState;

/// Represents the count statistics for a [Post](crate::posts::Post),
/// [Comment](crate::comment::Comment), etc.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Counts {
    pub id: Option<u64>,
    pub post_id: Option<u64>,
    pub comment_id: Option<u64>,
    pub comments: Option<u64>,
    pub score: i64,
    pub upvotes: u64,
    pub downvotes: u64,
    pub published: String,
    pub newest_comment_time_necro: Option<String>,
    pub newest_comment_time: Option<String>,
    pub featured_community: Option<bool>,
    pub featured_local: Option<bool>,
    pub hot_rank: Option<u64>,
    pub hot_rank_active: Option<u64>,
    pub child_count: Option<u64>,
}

impl Counts {
    /// Creates a new [Counts].
    pub const fn new() -> Counts {
        Self {
            id: None,
            post_id: None,
            comment_id: None,
            comments: None,
            score: 0,
            upvotes: 0,
            downvotes: 0,
            published: String::new(),
            newest_comment_time_necro: None,
            newest_comment_time: None,
            featured_community: None,
            featured_local: None,
            hot_rank: None,
            hot_rank_active: None,
            child_count: None,
        }
    }

    /// Gets the [Counts] ID.
    pub fn id(&self) -> u64 {
        self.id.clone().unwrap_or(0)
    }

    /// Gets whether the [Counts] are for a [Post](crate::posts::Post).
    pub fn for_post(&self) -> bool {
        self.post_id.is_some()
    }

    /// Gets the [Post](crate::posts::Post) ID for the [Counts].
    ///
    /// Returns `0` if the [Counts] is not for a [Post](crate::posts::Post).
    pub fn post_id(&self) -> u64 {
        self.post_id.unwrap_or(0)
    }

    /// Gets whether the [Counts] are for a [Comment](crate::comment::Comment).
    pub fn for_comment(&self) -> bool {
        self.comment_id.is_some()
    }

    /// Gets the [Comment](crate::comment::Comment) ID for the [Counts].
    ///
    /// Returns `0` if the [Counts] is not for a [Comment](crate::comment::Comment).
    pub fn comment_id(&self) -> u64 {
        self.comment_id.unwrap_or(0)
    }

    /// Gets the number of comments.
    pub fn comments(&self) -> u64 {
        self.comments.unwrap_or(0)
    }

    /// Gets the score [Counts] field.
    pub const fn score(&self) -> i64 {
        self.score
    }

    /// Gets the upvotes [Counts] field.
    pub const fn upvotes(&self) -> u64 {
        self.upvotes
    }

    /// Gets the downvotes [Counts] field.
    pub const fn downvotes(&self) -> u64 {
        self.downvotes
    }

    /// Gets the published date [Counts] field.
    pub fn published(&self) -> &str {
        self.published.as_str()
    }

    /// Gets the newest comment time necro date [Counts] field.
    ///
    /// Returns an empty string if the [Counts] is not for a [Post](crate::posts::Post].
    pub fn newest_comment_time_necro(&self) -> &str {
        self.newest_comment_time_necro.as_deref().unwrap_or("")
    }

    /// Gets the newest comment time date [Counts] field.
    ///
    /// Returns an empty string if the [Counts] is not for a [Post](crate::posts::Post].
    pub fn newest_comment_time(&self) -> &str {
        self.newest_comment_time.as_deref().unwrap_or("")
    }

    /// Gets the featured community [Counts] field.
    pub fn featured_community(&self) -> bool {
        self.featured_community.unwrap_or(false)
    }

    /// Gets the featured local [Counts] field.
    pub fn featured_local(&self) -> bool {
        self.featured_local.unwrap_or(false)
    }

    /// Gets the hot rank [Counts] field.
    pub fn hot_rank(&self) -> u64 {
        self.hot_rank.unwrap_or(0)
    }

    /// Gets the active hot rank [Counts] field.
    pub fn hot_rank_active(&self) -> u64 {
        self.hot_rank_active.unwrap_or(0)
    }

    /// Gets the child commments [Counts] field.
    pub fn child_count(&self) -> u64 {
        self.child_count.unwrap_or(0)
    }
}

/// List of [Counts] for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct Countss {
    pub items: Vec<Counts>,
    pub state: ListState,
}

impl Countss {
    /// Creates a new [Counts].
    pub fn new(items: Vec<Counts>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    /// Gets the list of [Counts] items.
    pub fn items(&self) -> &[Counts] {
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

    /// Gets an optional reference to the currently selected [Counts].
    pub fn current(&self) -> Option<&Counts> {
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

impl AsRef<Counts> for Counts {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Counts> for Counts {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
