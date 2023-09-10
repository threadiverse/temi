//! Facilities for drawing the PostsList screen.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time;

use crossterm::event;
use tui::{layout::Constraint, prelude::*, widgets::*};

use crate::{
    app::{App, TemiTerminal},
    posts::*,
    Result,
};

use super::{body_style, highlight_style, set_current_screen, title_block, Screen};

/// Draw the screen to show a list of [Posts](crate::posts::Posts).
pub fn draw_posts_screen(
    terminal: &mut TemiTerminal,
    app: &mut App,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    terminal.draw(|f| {
        let size = f.size();

        let frame_height = size.height as usize;

        let mut rows: Vec<Row> = app.posts.items.iter().map(|p| {
            let title = p.post.name.as_str();
            let author = p.creator.name();
            let date = p.creator.published();

            Row::new(vec![
                Cell::from(
                    Text::from(
                        vec![
                        Line::from(title),
                        Line::from(format!("    [ author: {author} | published: {date} ]")),
                        Line::from("-".repeat(size.width as usize)),
                        ]
                    )
                )
            ])
            .style(body_style())
            .height(3)
        })
        .collect();

        let total_height = rows.len() * 3;
        // add blank rows to push the info row(s) to the bottom
        for _ in total_height..(frame_height - 4) {
            rows.push(Row::new([""]));
        }

        rows.push(Row::new(["| (q) quit | (Enter) select | (◄, p) prev page | (▲)  prev post | (▼)  next post | next page (n, ►) |"]));

        let table = Table::new(rows)
            .style(body_style())
            .highlight_style(highlight_style())
            .column_spacing(0)
            .widths(&[Constraint::Percentage(100)])
            .block(title_block("Posts"));

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
