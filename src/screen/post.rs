//! Facilities for drawing the Post screen.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time;

use crossterm::event;
use tui::{prelude::*, widgets::scrollbar};

use crate::{
    app::{App, TemiTerminal},
    Result,
};

use super::{body_style, set_current_screen, title_block, Screen};

/// Draw the screen to show an individual [Post](crate::posts::Post).
pub fn draw_post_screen(
    terminal: &mut TemiTerminal,
    app: &mut App,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    terminal.draw(|f| {
        match app.posts.current() {
            Some(p) => {
                let size = f.size();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([
                                 Constraint::Percentage(30),
                                 Constraint::Percentage(60),
                                 Constraint::Percentage(5),
                                 Constraint::Percentage(5),
                    ]
                    .as_ref(),
                    )
                    .split(size);

                let creator = p.creator.name();
                let comments = p.counts.comments();
                let published = p.creator.published();

                let info = format!("creator: {creator}, published: {published}, comments: {comments}");

                let url = p.post.url();
                let title = p.post.name();
                let body = p.post.body();

                let max_width = [title.len(), info.len(), published.len(), body.len()]
                    .iter()
                    .max()
                    .map(|&m| m as u16)
                    .unwrap_or(body.len() as u16);

                let lines = vec![
                    Line::from(title),
                    Line::from(""),
                    Line::from(body),
                    Line::from(""),
                    Line::from(""),
                    Line::from(info),
                    Line::from(url),
                ];

                app.vertical_scroll_state = app.vertical_scroll_state.content_length(lines.len() as u16);
                app.horizontal_scroll_state = app.horizontal_scroll_state.content_length(max_width);

                let post_text = Paragraph::new(lines)
                    .style(body_style())
                    .block(title_block("Post"))
                    .wrap(Wrap { trim: false })
                    .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

                f.render_widget(post_text, chunks[0]);

                let post_scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));

                f.render_stateful_widget(
                    post_scrollbar,
                    chunks[0].inner(&Margin{ vertical: 1, horizontal: 0 }),
                    &mut app.vertical_scroll_state,
                );

                // multiple `Line`s per-comment for spacing/formatting
                let cap = app.comments[&p.post.id()].items.len() * 5;
                let mut comments: Vec<Line> = Vec::with_capacity(cap);

                if let Some(c) = app.comments.get_mut(&p.post.id()) {
                    c.items
                        .sort_by(|cr, cs| {
                            let cr_ids: Vec<u64> = cr.comment.path
                                .split('.')
                                .map(|p| p.parse::<u64>().unwrap_or(0))
                                .collect();

                            let cs_ids: Vec<u64> = cs.comment.path
                                .split('.')
                                .map(|p| p.parse::<u64>().unwrap_or(0))
                                .collect();

                            let cr_first = cr_ids.get(1).unwrap_or(&0);
                            let cs_first = cs_ids.get(1).unwrap_or(&0);

                            // FIXME: this is not a proper sort, we need multiple passes
                            //
                            // first, to group all the comments with the same parent.
                            // then, to group all the comments with the same first child.
                            // and so on, until the end depth is reached.
                            // then each child comment group needs to be sorted earliest to latest.
                            // ... chronology and sorting is hard ...
                            //
                            // this probably needs to be a recursive function that takes comment
                            // depth as a parameter...
                            cr_first.cmp(cs_first).then_with(|| cr_ids.len().cmp(&cs_ids.len()))
                        });

                    for cr in c.items.iter() {
                        let ct = cr.comment.content();
                        let a = cr.creator.name();
                        let n = cr.counts.child_count();
                        let tabs = "►".repeat(cr.comment.path.split('.').count().saturating_sub(2));
                        let blocks = "█".repeat(cr.comment.path.split('.').count().saturating_sub(2));

                        let info = format!("[ author: {a}, child comments: {n} ]");

                        // FIXME: comment indentation, child comments should be indented by depth
                        // e.g. <parent>.<child0>, one level indent
                        //      <parent>.<child0>.<subchild0> two level indent, etc.
                        comments.push(Line::from(vec![Span::raw(tabs.clone()), Span::raw(" "), Span::raw(ct)]));
                        comments.push(Line::from(""));
                        comments.push(Line::from(vec![Span::raw(blocks), Span::raw(" "), Span::raw(info)]));
                        comments.push(Line::from(""));
                        comments.push(Line::from(""));
                    }
                }

                let comment_block = Paragraph::new(comments)
                    .style(body_style())
                    .block(title_block("Comments"))
                    .wrap(Wrap { trim: false })
                    .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

                f.render_widget(comment_block, chunks[1]);

                let comment_scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));

                f.render_stateful_widget(
                    comment_scrollbar,
                    chunks[0].inner(&Margin{ vertical: 1, horizontal: 0 }),
                    &mut app.vertical_scroll_state,
                );

                let hud = Block::default()
                    .title("| (q) quit | (Enter) select | (◄, ▲, ▼, ►) scroll | (n) next | (p) previous |")
                    .title_alignment(Alignment::Right);

                f.render_widget(hud, chunks[3]);
            }
            _ => set_current_screen(Screen::PostList),
        }
    })?;

    if event::poll(time::Duration::from_millis(200))? {
        if let event::Event::Key(event) = event::read()? {
            match event.code {
                event::KeyCode::Esc => set_current_screen(Screen::PostList),
                event::KeyCode::Enter => set_current_screen(Screen::CommentList),
                event::KeyCode::Up => {
                    app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
                    app.vertical_scroll_state = app
                        .vertical_scroll_state
                        .position(app.vertical_scroll as u16);
                }
                event::KeyCode::Down => {
                    app.vertical_scroll = app.vertical_scroll.saturating_add(1);
                    app.vertical_scroll_state = app
                        .vertical_scroll_state
                        .position(app.vertical_scroll as u16);
                }
                event::KeyCode::Left => {
                    app.horizontal_scroll = app.horizontal_scroll.saturating_sub(1);
                    app.horizontal_scroll_state = app
                        .horizontal_scroll_state
                        .position(app.horizontal_scroll as u16);
                }
                event::KeyCode::Right => {
                    app.horizontal_scroll = app.horizontal_scroll.saturating_add(1);
                    app.horizontal_scroll_state = app
                        .horizontal_scroll_state
                        .position(app.horizontal_scroll as u16);
                }
                event::KeyCode::Char('n') => app.posts.next(),
                event::KeyCode::Char('p') => app.posts.previous(),
                event::KeyCode::Char('i') => set_current_screen(Screen::Image),
                event::KeyCode::Char('c') => {
                    if event.modifiers == event::KeyModifiers::CONTROL {
                        stop.store(true, Ordering::SeqCst);
                    }
                }
                event::KeyCode::Char('q') => stop.store(true, Ordering::SeqCst),
                _ => (),
            }
        }
    }

    Ok(())
}
