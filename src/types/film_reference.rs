//! A reference to a film, as it appears inside a feed item.

use serde::{Deserialize, Serialize};

use crate::types::TmdbId;

/// A reference to a film, as it appears inside a feed item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmReference {
    /// The film's title.
    pub title: String,
    /// The film's release year.
    pub year: Option<i32>,
    /// The film's canonical Letterboxd URL.
    pub url: Option<String>,
    /// The film's TMDB identifier.
    pub tmdb_id: Option<TmdbId>,
}
