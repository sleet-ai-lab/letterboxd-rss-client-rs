//! A single film inside a curated list.

use serde::{Deserialize, Serialize};

/// A single film inside a [`LetterboxdList`](crate::types::LetterboxdList).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListFilm {
    /// The film's title as shown on Letterboxd.
    pub title: String,
    /// The film's canonical Letterboxd URL.
    pub url: Option<String>,
    /// The position of the film in the list, starting at `1`.
    pub position: u32,
}
