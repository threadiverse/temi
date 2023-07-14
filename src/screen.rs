use std::sync::atomic::{AtomicU16, Ordering};

use tui::{prelude::*, style::Style};

mod post;
mod posts_list;

pub use post::*;
pub use posts_list::*;

/// Convenience definition for purple color style.
pub const PURPLE: Color = Color::Rgb(0x80, 0x00, 0x80);
/// Convenience definition for gray color style.
pub const GRAY: Color = Color::Rgb(0xbf, 0xba, 0xbe);
/// Convenience definition for dark gray color style.
pub const DARK_GRAY: Color = Color::Rgb(0x3f, 0x3a, 0x3e);
/// Convenience definition for white smoke color style.
pub const WHITE_SMOKE: Color = Color::Rgb(0xf5, 0xf5, 0xf5);

static CURRENT_SCREEN: AtomicU16 = AtomicU16::new(0);

/// Gets the currently set [Screen] to display.
pub fn current_screen() -> Screen {
    CURRENT_SCREEN.load(Ordering::Relaxed).into()
}

/// Sets the [Screen] to display.
pub fn set_current_screen(screen: Screen) {
    CURRENT_SCREEN.store(screen.into(), Ordering::SeqCst);
}

/// Representation of the selected screen.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Screen {
    #[default]
    PostList = 0,
    Post,
    Image,
    CommentList,
    Comment,
}

impl From<u16> for Screen {
    fn from(val: u16) -> Self {
        match val {
            0 => Self::PostList,
            1 => Self::Post,
            2 => Self::Image,
            3 => Self::CommentList,
            4 => Self::Comment,
            _ => Self::PostList,
        }
    }
}

impl From<usize> for Screen {
    fn from(val: usize) -> Screen {
        (val as u16).into()
    }
}

impl From<&Screen> for u16 {
    fn from(val: &Screen) -> Self {
        *val as u16
    }
}

impl From<Screen> for u16 {
    fn from(val: Screen) -> Self {
        (&val).into()
    }
}

/// Creates a title block
pub fn title_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .style(header_style())
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

/// Gets the default style for displaying a [Screen].
pub fn screen_style() -> Style {
    Style::default().fg(PURPLE).bg(Color::Black)
}

/// Gets the default style for displaying a header.
pub fn header_style() -> Style {
    Style::default().fg(WHITE_SMOKE).bg(Color::Black)
}

/// Gets the default style for displaying a list.
pub fn list_style() -> Style {
    Style::default().fg(Color::Green).bg(Color::Black)
}

/// Gets the default style for displaying the body of a table.
pub fn body_style() -> Style {
    Style::default().fg(Color::Green).bg(Color::Black)
}

/// Gets the default style for highlighting.
pub fn highlight_style() -> Style {
    Style::default().fg(PURPLE).bg(GRAY)
}

/// Split text into cell width, useful for table layouts that have
/// text that needs to span multiple cells.
///
/// Currently, only works for evenly spaced cells.
///
/// Returns the total height of the row.
pub fn split_cells(text: &str, width: usize, out: &mut [String]) -> usize {
    let mut cell_idx = 0;
    let num_cells = out.len();
    let mut height = 1;

    let stripped: String = text.chars().filter(|&c| c != '\r' && c != '\n').collect();

    for c in stripped.as_bytes().chunks(width) {
        if height != 1 {
            out[cell_idx] += format!("\n{}", std::str::from_utf8(c).unwrap_or("")).as_str();
        } else {
            out[cell_idx] += std::str::from_utf8(c).unwrap_or("");
        }

        cell_idx = (cell_idx + 1) % num_cells;

        if cell_idx == 0 {
            height += 1;
        }
    }

    height
}
