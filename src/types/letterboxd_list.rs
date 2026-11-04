//! A curated list of films published by a member.

use serde::{Deserialize, Serialize};

use crate::types::ListFilm;

/// A member-curated list of films.
///
/// Letterboxd renders the first few entries of a list into the description
/// HTML and appends a "…plus N more" link. [`LetterboxdList::films`] therefore
/// holds only the films that were inlined in the feed, which is why
/// [`LetterboxdList::truncated_count`] is exposed alongside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetterboxdList {
    /// The GUID, of the form `letterboxd-list-<id>`.
    pub guid: String,
    /// The raw `<title>` text.
    pub title: String,
    /// The raw `<link>` URL pointing at the list page.
    pub link: String,
    /// The raw `<pubDate>` string, which is when the list was last updated.
    pub pub_date: Option<String>,
    /// The member's display name.
    pub author: Option<String>,
    /// The list's description as plain text.
    pub description_text: Option<String>,
    /// The list's description as raw HTML.
    pub description_html: Option<String>,
    /// The films inlined in the feed description, in list order.
    pub films: Vec<ListFilm>,
    /// The number of films omitted from the feed, when the list was truncated.
    pub truncated_count: Option<u64>,
}

impl LetterboxdList {
    /// Returns the number of films that were inlined in the feed.
    pub fn visible_count(&self) -> usize {
        self.films.len()
    }

    /// Returns the total number of films on the list, when known.
    pub fn total_count(&self) -> Option<u64> {
        self.truncated_count
            .map(|extra| extra + self.films.len() as u64)
    }

    /// Returns whether the feed only carried part of the list.
    pub fn is_truncated(&self) -> bool {
        self.truncated_count.unwrap_or(0) > 0
    }
}
