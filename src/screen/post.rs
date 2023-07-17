//! Facilities for drawing the Post screen.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time;

use crossterm::event;
use tui::{prelude::*, widgets::scrollbar};

use crate::{
    app::{App, Scroll, TemiTerminal},
    Result,
};

use super::{body_style, set_current_screen, title_block, wrapped_height, Screen};

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
                                 Constraint::Min(1),
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

                let post_lens = [title.len(), info.len(), published.len(), body.len()];

                let lines = vec![
                    Line::from(title),
                    Line::from(""),
                    Line::from(body),
                    Line::from(""),
                    Line::from(""),
                    Line::from(info),
                    Line::from(url),
                ];

                let posts_height: usize = wrapped_height(post_lens.iter().sum(), size.width as usize);
                app.post_scroll.set_content_length(posts_height as u16);

                let post_text = Paragraph::new(lines)
                    .style(body_style())
                    .block(title_block("Post"))
                    .wrap(Wrap { trim: false })
                    .scroll((app.post_scroll.position(), 0));

                f.render_widget(post_text, chunks[0]);

                let orientation = ScrollbarOrientation::VerticalRight;
                let post_scrollbar = Scrollbar::default()
                    .orientation(orientation.clone())
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));

                f.render_stateful_widget(
                    post_scrollbar,
                    chunks[0].inner(&Scroll::margin()),
                    &mut app.post_scroll.state,
                );

                // multiple `Line`s per-comment for spacing/formatting
                let cap = app.comments[&p.post.id()].items.len() * 5;
                let mut comments: Vec<Line> = Vec::with_capacity(cap);

                let mut comment_height = 0;
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
                        let blocks = "_".repeat(cr.comment.path.split('.').count().saturating_sub(2));

                        let info = format!("[ author: {a}, child comments: {n} ]");

                        let height = ct.len() + a.len() + tabs.len() + blocks.len() + info.len();
                        comment_height += wrapped_height(height, size.width as usize) + 2;

                        // FIXME: comment indentation, child comments should be indented by depth
                        // e.g. <parent>.<child0>, one level indent
                        //      <parent>.<child0>.<subchild0> two level indent, etc.
                        comments.push(Line::from(vec![Span::raw(tabs.clone()), Span::raw(" "), Span::raw(ct)]));
                        comments.push(Line::from(vec![Span::raw(blocks), Span::raw(" "), Span::raw(info)]));
                        comments.push(Line::from(""));
                        comments.push(Line::from(""));
                    }
                }

                app.comment_scroll.set_content_length(comment_height as u16);

                let comment_block = Paragraph::new(comments)
                    .style(body_style())
                    .block(title_block("Comments"))
                    .wrap(Wrap { trim: false })
                    .scroll((app.comment_scroll.position(), 0));

                f.render_widget(comment_block, chunks[1]);

                let comment_scrollbar = Scrollbar::default()
                    .orientation(orientation.clone())
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));

                f.render_stateful_widget(
                    comment_scrollbar,
                    chunks[1].inner(&Scroll::margin()),
                    &mut app.comment_scroll.state,
                );

                let hud = Block::default()
                    .title("| (q) quit | (Enter) select | (▲, ▼) scroll post | (j, k) scroll comment | (n) next | (p) previous |")
                    .title_alignment(Alignment::Right);

                f.render_widget(hud, chunks[4]);
            }
            _ => set_current_screen(Screen::PostList),
        }
    })?;

    if event::poll(time::Duration::from_millis(200))? {
        if let event::Event::Key(event) = event::read()? {
            match event.code {
                event::KeyCode::Esc => set_current_screen(Screen::PostList),
                event::KeyCode::Enter => set_current_screen(Screen::CommentList),
                event::KeyCode::Up => app.post_scroll.prev(),
                event::KeyCode::Down => app.post_scroll.next(),
                event::KeyCode::Char('k') => app.comment_scroll.prev(),
                event::KeyCode::Char('j') => app.comment_scroll.next(),
                event::KeyCode::Char('n') => {
                    app.post_scroll.first();
                    app.comment_scroll.first();

                    app.posts.next()
                }
                event::KeyCode::Char('p') => {
                    app.post_scroll.first();
                    app.comment_scroll.first();

                    app.posts.previous()
                }
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
