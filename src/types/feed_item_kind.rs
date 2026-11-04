//! The three kinds of content Letterboxd puts in an RSS feed.

use serde::{Deserialize, Serialize};

/// The three kinds of content Letterboxd puts in an RSS feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedItemKind {
    /// A watched-film diary entry.
    Diary,
    /// A curated list of films.
    List,
    /// A review that the member liked.
    LikedReview,
}

impl FeedItemKind {
    /// Returns the GUID prefix Letterboxd uses for this kind.
    pub fn guid_prefix(&self) -> &'static str {
        match self {
            FeedItemKind::Diary => "letterboxd-watch",
            FeedItemKind::List => "letterboxd-list",
            FeedItemKind::LikedReview => "letterboxd-like",
        }
    }
}
