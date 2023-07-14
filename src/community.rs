use tui::widgets::ListState;

/// Represents a response to an API request that presents a `community` field.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Community {
    pub id: u64,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub removed: bool,
    pub published: String,
    pub updated: Option<String>,
    pub deleted: bool,
    pub nsfw: bool,
    pub actor_id: String,
    pub local: bool,
    pub icon: Option<String>,
    pub hidden: bool,
    pub posting_restricted_to_mods: bool,
    pub instance_id: u64,
}

impl Community {
    /// Creates a new [Community].
    pub const fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            title: String::new(),
            description: None,
            removed: false,
            published: String::new(),
            updated: None,
            deleted: false,
            nsfw: false,
            actor_id: String::new(),
            local: false,
            icon: None,
            hidden: false,
            posting_restricted_to_mods: false,
            instance_id: 0,
        }
    }
}

/// List of [Community] for displaying in the TUI.
#[derive(Clone, Debug)]
pub struct Communities {
    pub items: Vec<Community>,
    pub state: ListState,
}

impl Communities {
    /// Creates a new [Communities].
    pub fn new(items: Vec<Community>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    /// Gets the list of [Community] items.
    pub fn items(&self) -> &[Community] {
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

    /// Gets an optional reference to the currently selected [Community].
    pub fn current(&self) -> Option<&Community> {
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

impl AsRef<Communities> for Communities {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Communities> for Communities {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
