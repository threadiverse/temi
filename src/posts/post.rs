use tui::widgets::ListState;

/// Represents a post as returned in a posts API response.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Post {
    pub id: u64,
    pub name: String,
    pub url: Option<String>,
    pub deleted: bool,
    pub nsfw: bool,
    pub thumbnail_url: Option<String>,
    pub ap_id: String,
    pub body: Option<String>,
    pub sorted: Option<bool>,
}

impl Post {
    /// Creates a new [Post].
    pub const fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            url: None,
            deleted: false,
            nsfw: false,
            thumbnail_url: None,
            ap_id: String::new(),
            body: None,
            sorted: None,
        }
    }

    /// Gets the [Post] name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the [Post] ID.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Gets the [Post] URL.
    pub fn url(&self) -> &str {
        self.url.as_deref().unwrap_or("")
    }

    /// Gets the [Post] body.
    pub fn body(&self) -> &str {
        self.body.as_deref().unwrap_or("")
    }

    /// Gets the [Post] thumbnail URL.
    pub fn thumbnail_url(&self) -> &str {
        self.thumbnail_url.as_deref().unwrap_or("")
    }

    /// Gets the [Post] AP ID.
    pub fn ap_id(&self) -> &str {
        self.ap_id.as_str()
    }

    pub fn sorted(&self) -> bool {
        matches!(self.sorted, Some(true))
    }

    pub fn set_sorted(&mut self, val: bool) {
        self.sorted = Some(val);
    }

    pub fn unset_sorted(&mut self) {
        self.sorted.take();
    }
}

/// List of [Post]s for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct Posts {
    pub items: Vec<Post>,
    pub state: ListState,
}

impl Posts {
    /// Creates a new [Posts].
    pub fn new(items: Vec<Post>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    /// Gets the list of [Post] items.
    pub fn items(&self) -> &[Post] {
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

    /// Gets an optional reference to the currently selected [Post].
    pub fn current(&self) -> Option<&Post> {
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

impl AsRef<Posts> for Posts {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Posts> for Posts {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
