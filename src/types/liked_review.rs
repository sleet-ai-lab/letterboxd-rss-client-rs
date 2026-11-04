//! A review by another member that the feed owner liked.

use serde::{Deserialize, Serialize};

use crate::types::FilmReference;

/// A review by another member that the feed owner liked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikedReview {
    /// The GUID, of the form `letterboxd-like-<id>`.
    pub guid: String,
    /// The raw `<title>` text.
    pub title: String,
    /// The raw `<link>` URL.
    pub link: String,
    /// The raw `<pubDate>` string.
    pub pub_date: Option<String>,
    /// The review author's display name.
    pub author: Option<String>,
    /// The film the review is about.
    pub film: Option<FilmReference>,
    /// The review body as plain text, when present.
    pub review_text: Option<String>,
    /// The review body as raw HTML, when present.
    pub review_html: Option<String>,
}
