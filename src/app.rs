use std::sync::atomic::{AtomicBool, Ordering};

use hashbrown::HashMap;

use tui::{backend::CrosstermBackend, prelude::*, Terminal};

use crate::{comments::CommentResponseTable, posts::PostResponseTable};

static REFRESH: AtomicBool = AtomicBool::new(false);

pub fn refresh() -> bool {
    REFRESH.load(Ordering::Relaxed)
}

pub fn set_refresh(val: bool) {
    REFRESH.store(val, Ordering::SeqCst);
}

/// Convenience alias for the [Terminal](tui::Terminal) type used in `temi`.
pub type TemiTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

/// Represents the application state.
pub struct App {
    pub instance_url: String,
    pub page: u64,
    pub posts: PostResponseTable,
    pub comments: HashMap<u64, CommentResponseTable>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
}

impl App {
    /// Creates a new [App] instance.
    pub fn new(instance_url: String, posts: PostResponseTable) -> Self {
        Self {
            instance_url,
            page: 1,
            posts,
            comments: HashMap::new(),
            vertical_scroll_state: ScrollbarState::default(),
            horizontal_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
        }
    }

    /// Gets the current [PostList](crate::endpoint::Endpoint) endpoint page.
    pub fn page(&self) -> u64 {
        self.page
    }

    /// Increments the page number.
    pub fn next_page(&mut self) -> u64 {
        self.page = self.page.saturating_add(1);
        self.page
    }

    /// Decrements the page number.
    pub fn previous_page(&mut self) -> u64 {
        if self.page > 1 {
            self.page = self.page.saturating_sub(1);
        }
        self.page
    }
}

impl AsRef<App> for App {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<App> for App {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
