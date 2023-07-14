//! Endpoint definitions.

use std::fmt;

/// Represents the different API endpoints.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Endpoint {
    #[default]
    PostList,
    CommentList,
}

impl From<Endpoint> for &'static str {
    fn from(val: Endpoint) -> Self {
        match val {
            Endpoint::PostList => "/api/v3/post/list",
            Endpoint::CommentList => "/api/v3/comment/list",
        }
    }
}

impl From<&Endpoint> for &'static str {
    fn from(val: &Endpoint) -> Self {
        (*val).into()
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", <&str>::from(self))
    }
}
