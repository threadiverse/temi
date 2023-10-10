//! Types and functions for posts.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use tui::widgets::TableState;

use crate::{counts::Counts, utils::write_to_file, Error, Result};

mod creator;
mod post;

pub use creator::{Creator, Creators};
pub use post::{Post, Posts};

static DOWNLOAD_POSTS: AtomicBool = AtomicBool::new(false);

/// Gets whether to download posts.
pub fn download_posts() -> bool {
    DOWNLOAD_POSTS.load(Ordering::Relaxed)
}

/// Sets whether to download posts.
pub fn set_download_posts(val: bool) {
    DOWNLOAD_POSTS.store(val, Ordering::SeqCst)
}

/// Download a response to the [PostList](crate::endpoint::Endpoint) endpoint.
pub async fn dl_posts(url: &str) -> Result<PostResponses> {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let response = client.get(hyper::Uri::from_str(url)?).await?;

    let body = hyper::body::to_bytes(response.into_body()).await?;

    #[cfg(feature = "debug_endpoints")]
    write_to_file("posts.json", &body)?;

    serde_json::from_slice::<PostResponses>(&body).map_err(|err| err.into())
}

/// Gets whether the URL points to an image file.
pub fn is_image(url: &str) -> bool {
    url.ends_with(".bmp")
        || url.ends_with(".gif")
        || url.ends_with(".jpg")
        || url.ends_with(".jpeg")
        || url.ends_with(".png")
        || url.ends_with(".webp")
}

/// Gets the temporary file name for an image.
// FIXME: stream the image without creating a temporary file.
pub fn image_name(url: &str) -> Result<&str> {
    let uri = url.parse::<http::Uri>()?;
    let path = uri.path();

    if path.ends_with(".bmp") {
        Ok("tmp.bmp")
    } else if path.ends_with(".gif") {
        Ok("tmp.gif")
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Ok("tmp.jpeg")
    } else if path.ends_with(".png") {
        Ok("tmp.png")
    } else if path.ends_with(".webp") {
        Ok("tmp.webp")
    } else {
        Err(Error::Image("unsupported image type".into()))
    }
}

/// Download a [Post](crate::posts::Post) image.
pub async fn dl_image(url: &str, file_name: &str) -> Result<()> {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let response = client.get(hyper::Uri::from_str(url)?).await?;

    let body = hyper::body::to_bytes(response.into_body()).await?;

    write_to_file(file_name, &body)?;

    Ok(())
}

/// Load posts from a file instead of making a call to an endpoint.
///
/// Avoids pinging an API endpoint, and needlessly overloading a server.
///
/// Let's be nice to our friendly Lemmy instances :)
pub fn load_posts(file_name: &str) -> Result<PostResponses> {
    use std::io::Read;

    let mut file = std::fs::File::open(file_name)?;
    let mut res = String::new();

    file.read_to_string(&mut res)?;

    serde_json::from_str::<PostResponses>(res.as_str()).map_err(|err| err.into())
}

/// Represents a response from the [Post endpoint](crate::endpoint::Endpoint).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PostResponse {
    pub post: Post,
    pub creator: Creator,
    pub counts: Counts,
}

/// Represents a list of responses to the [Post endpoint](crate::endpoint::Endpoint).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PostResponses {
    pub posts: Vec<PostResponse>,
}

/// Represents a table of responses to the [Post endpoint](crate::endpoint::Endpoint).
pub struct PostResponseTable {
    pub items: Vec<PostResponse>,
    pub state: TableState,
}

impl PostResponseTable {
    /// Creates a new [PostResponseTable].
    pub fn new(items: Vec<PostResponse>) -> Self {
        Self {
            items,
            state: TableState::default(),
        }
    }

    /// Gets the list of [Post] items.
    pub fn items(&self) -> &[PostResponse] {
        self.items.as_ref()
    }

    /// Gets a reference to the current [TableState].
    pub fn state(&self) -> &TableState {
        &self.state
    }

    /// Gets a mutable reference to the current [TableState].
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    /// Gets an optional reference to the currently selected [PostResponse].
    pub fn current(&self) -> Option<&PostResponse> {
        if let Some(i) = self.state.selected() {
            self.items.get(i)
        } else {
            None
        }
    }

    /// Gets an optional mutable reference to the currently selected [PostResponse].
    pub fn current_mut(&mut self) -> Option<&mut PostResponse> {
        if let Some(i) = self.state.selected() {
            self.items.get_mut(i)
        } else {
            None
        }
    }

    /// Clears the [TableState] selection.
    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    /// Updates the [TableState] to select the next item.
    pub fn next(&mut self) {
        let len = self.items.len();
        let i = self.state.selected().map(|i| (i + 1) % len).unwrap_or(0);
        self.state.select(Some(i));
    }

    /// Updates the [TableState] to select the previous item.
    pub fn previous(&mut self) {
        let len = self.items.len();
        let last = len.saturating_sub(1);
        let i = self
            .state
            .selected()
            .map(|i| if i == 0 { last } else { i.saturating_sub(1) })
            .unwrap_or(last);
        self.state.select(Some(i));
    }
}

impl From<Vec<PostResponse>> for PostResponseTable {
    fn from(val: Vec<PostResponse>) -> Self {
        Self::new(val)
    }
}

impl From<PostResponses> for PostResponseTable {
    fn from(val: PostResponses) -> Self {
        Self::new(val.posts)
    }
}

impl AsRef<PostResponse> for PostResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<PostResponse> for PostResponse {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
