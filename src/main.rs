use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use temi::{app::*, comments::*, endpoint::*, posts::*, screen::*, Result};

#[tokio::main]
async fn main() -> Result<()> {
    terminal::enable_raw_mode()?;

    let stop = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&stop))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let instance_url = std::env::var("LEMMY_INSTANCE").unwrap_or("https://voyager.lemmy.ml".into());

    let post_ep = Endpoint::PostList;
    let comment_ep = Endpoint::CommentList;

    let posts_res = dl_posts(format!("{instance_url}{post_ep}?page=1").as_str()).await?;
    let posts = PostResponseTable::from(posts_res);

    let mut app = App::new(instance_url, posts);

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        match current_screen() {
            Screen::Post => {
                if let Some(post) = app.posts.current() {
                    let post_id = post.post.id();
                    let num_comments = post.counts.comments() as usize;

                    if app.comments.get(&post_id).is_none() || refresh() {
                        let instance_url = app.instance_url.as_str();
                        let mut responses = CommentResponses::new(Vec::with_capacity(num_comments));

                        for page in 0..(num_comments / 50) {
                            let page = page + 1;
                            let comment_url = format!(
                                "{instance_url}{comment_ep}?post_id={post_id}&page={page}&limit=50"
                            );
                            responses
                                .comments
                                .append(&mut dl_comments(comment_url.as_str()).await?.comments);
                        }

                        if num_comments % 50 > 0 {
                            let page = (num_comments / 50) + 1;
                            let comment_url = format!(
                                "{instance_url}{comment_ep}?post_id={post_id}&page={page}&limit=50"
                            );
                            responses
                                .comments
                                .append(&mut dl_comments(comment_url.as_str()).await?.comments);
                        }

                        app.comments.remove(&post_id);
                        app.comments.insert(post_id, responses.into());

                        set_refresh(false);
                    }

                    draw_post_screen(&mut terminal, app.as_mut(), Arc::clone(&stop))?;
                } else {
                    set_current_screen(Screen::PostList);
                }
            }
            Screen::PostList => {
                if download_posts() {
                    let instance_url = app.instance_url.as_str();
                    let page = app.page();

                    app.posts = dl_posts(format!("{instance_url}{post_ep}?page={page}").as_str())
                        .await?
                        .into();

                    set_download_posts(false);
                }
                draw_posts_screen(&mut terminal, app.as_mut(), Arc::clone(&stop))?
            }
            _ => (),
        }
    }

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
