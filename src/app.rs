use std::sync::atomic::{AtomicBool, Ordering};

use hashbrown::HashMap;

use tui::{backend::CrosstermBackend, prelude::*, widgets::*, Terminal};

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

/// State for a scrollbar.
#[derive(Clone, Default)]
pub struct Scroll {
    pub state: ScrollbarState,
    pub position: u16,
    pub content_length: u16,
    pub viewport_length: u16,
}

impl Scroll {
    /// Creates a new [Scroll].
    pub fn new() -> Self {
        Self {
            state: ScrollbarState::default(),
            position: 0,
            content_length: 0,
            viewport_length: 0,
        }
    }

    /// Creates a new [Scroll] with the provided parameters.
    pub fn create(position: u16, content_length: u16, viewport_length: u16) -> Self {
        Self {
            state: ScrollbarState::default()
                .position(position as usize)
                .content_length(content_length as usize)
                .viewport_content_length(viewport_length as usize),
            position,
            content_length,
            viewport_length,
        }
    }

    /// Gets the [Scroll] position.
    pub fn position(&self) -> u16 {
        self.position
    }

    /// Sets the [Scroll] position.
    pub fn set_position(&mut self, pos: u16) {
        self.position = pos;
        self.state = self.state.position(pos as usize);
    }

    /// Gets the [Scroll] content length.
    pub fn content_length(&self) -> u16 {
        self.content_length
    }

    /// Sets the [Scroll] content length.
    pub fn set_content_length(&mut self, len: u16) {
        self.content_length = len;
        self.state = self.state.content_length(len as usize);
    }

    /// Gets the [Scroll] viewport length.
    pub fn viewport_length(&self) -> u16 {
        self.viewport_length
    }

    /// Sets the [Scroll] viewport length.
    pub fn set_viewport_length(&mut self, len: u16) {
        self.viewport_length = len;

        self.state = self.state.viewport_content_length(len as usize);
    }

    /// Gets the [Margin] from the [Scroll] position.
    pub fn margin() -> Margin {
        Margin {
            vertical: 0,
            horizontal: 0,
        }
    }

    /// Moves to the next scrollbar position.
    pub fn next(&mut self) {
        self.position = self
            .position
            .saturating_add(1)
            .clamp(0, self.content_length.saturating_sub(1));

        self.state.next();
    }

    /// Moves to the previous scrollbar position.
    pub fn prev(&mut self) {
        self.position = self.position.saturating_sub(1);

        self.state.prev();
    }

    /// Moves to the first scrollbar position.
    pub fn first(&mut self) {
        self.position = 0;
        self.state.first();
    }

    /// Moves to the last scrollbar position.
    pub fn last(&mut self) {
        self.position = self.content_length.saturating_sub(1);
        self.state.last();
    }
}

/// Represents the application state.
pub struct App {
    pub instance_url: String,
    pub page: u64,
    pub posts: PostResponseTable,
    pub comments: HashMap<u64, CommentResponseTable>,
    pub post_scroll: Scroll,
    pub comment_scroll: Scroll,
}

impl App {
    /// Creates a new [App] instance.
    pub fn new(instance_url: String, posts: PostResponseTable) -> Self {
        Self {
            instance_url,
            page: 1,
            posts,
            comments: HashMap::new(),
            post_scroll: Scroll::new(),
            comment_scroll: Scroll::new(),
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
