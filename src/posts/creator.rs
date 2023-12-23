use tui::widgets::ListState;

/// Represents a post creator as returned in a posts API response.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Creator {
    pub id: u64,
    pub name: String,
    pub avatar: Option<String>,
    pub banned: bool,
    pub published: String,
    pub actor_id: String,
    pub local: bool,
    pub icon: Option<String>,
    pub deleted: bool,
    pub admin: Option<bool>,
    pub bot_account: bool,
    pub instance_id: u64,
}

impl Creator {
    /// Creates a new [Creator].
    pub const fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            avatar: None,
            banned: false,
            published: String::new(),
            actor_id: String::new(),
            local: false,
            icon: None,
            deleted: false,
            admin: None,
            bot_account: false,
            instance_id: 0,
        }
    }

    /// Gets the [Creator] name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the [Creator] published date.
    pub fn published(&self) -> &str {
        self.published.as_str()
    }

    /// Gets the [Creator] icon URL.
    pub fn icon(&self) -> &str {
        self.icon.as_deref().unwrap_or("")
    }

    /// Gets the [Creator] avatar URL.
    pub fn avatar(&self) -> &str {
        self.avatar.as_deref().unwrap_or("")
    }
}

/// List of [Creator]s for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct Creators {
    pub items: Vec<Creator>,
    pub state: ListState,
}

impl Creators {
    /// Creates a new [Creators].
    pub fn new(items: Vec<Creator>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    /// Gets the list of [Creator] items.
    pub fn items(&self) -> &[Creator] {
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

    /// Gets an optional reference to the currently selected [Creator].
    pub fn current(&self) -> Option<&Creator> {
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
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() {
                    0
                } else {
                    i + 1
                }
            }
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

impl AsRef<Creator> for Creator {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Creator> for Creator {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
