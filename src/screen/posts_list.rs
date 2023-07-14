//! Facilities for drawing the PostsList screen.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time;

use crossterm::event;
use tui::{
    layout::Constraint,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    app::{App, TemiTerminal},
    posts::*,
    screen::Screen,
    Result,
};

use super::{body_style, highlight_style, screen_style, set_current_screen, split_cells};

/// Draw the screen to show a list of [Posts](crate::posts::Posts).
pub fn draw_posts_screen(
    terminal: &mut TemiTerminal,
    app: &mut App,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    terminal.draw(|f| {
        let block = Block::default()
            .title("temi : Posts")
            .borders(Borders::ALL)
            .style(screen_style());

        let size = f.size();

        let frame_height = size.height as usize;
        let cell_width = (size.width as f32 * 0.10) as usize;

        let mut total_height = 1;
        let mut rows: Vec<Row> = app
            .posts
            .items
            .iter()
            .map(|p| {
                let mut cell_text = [
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ];

                let row_height = split_cells(p.post.name.as_str(), cell_width, &mut cell_text);

                total_height += row_height + 1;

                Row::new(cell_text)
                    .style(body_style())
                    .height(row_height as u16)
                    .bottom_margin(1)
            })
            .collect();

        // add blank rows to push the info row(s) to the bottom
        for _ in total_height..(frame_height - 4) {
            rows.push(Row::new([""]).height(1).bottom_margin(0));
        }

        rows.push(
            Row::new([
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(" | (q) quit"),
                Cell::from(" | (Enter) select"),
                Cell::from(" | (◄, p) prev page"),
                Cell::from(" | (▲)  prev post"),
                Cell::from(" | (▼)  next post"),
                Cell::from(" | next page (n, ►) |"),
                Cell::from(""),
            ])
            .style(body_style())
            .height(1)
            .bottom_margin(0),
        );

        let widths = [
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ];

        let table = Table::new(rows)
            .style(body_style())
            .widths(&widths)
            .highlight_style(highlight_style())
            .column_spacing(0)
            .block(block);

        f.render_stateful_widget(table, size, &mut app.posts.state);
    })?;

    if event::poll(time::Duration::from_millis(200))? {
        if let event::Event::Key(event) = event::read()? {
            match event.code {
                event::KeyCode::Esc => app.posts.deselect(),
                event::KeyCode::Down => app.posts.next(),
                event::KeyCode::Up => app.posts.previous(),
                event::KeyCode::Enter => set_current_screen(Screen::Post),
                event::KeyCode::Char('c') => {
                    if event.modifiers == event::KeyModifiers::CONTROL {
                        stop.store(true, Ordering::SeqCst);
                    }
                }
                event::KeyCode::Char('n') | event::KeyCode::Right => {
                    app.next_page();
                    set_download_posts(true);
                }
                event::KeyCode::Char('p') | event::KeyCode::Left => {
                    app.previous_page();
                    set_download_posts(true);
                }
                event::KeyCode::Char('q') => stop.store(true, Ordering::SeqCst),
                _ => (),
            }
        }
    }

    Ok(())
}
