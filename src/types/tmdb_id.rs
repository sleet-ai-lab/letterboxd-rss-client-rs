//! A TMDB identifier for a film.

use serde::{Deserialize, Serialize};

/// A TMDB identifier for a film.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TmdbId {
    /// The film is a movie, with a TMDB movie id.
    Movie(u64),
    /// The film is a TV series, with a TMDB TV id.
    Tv(u64),
}

impl TmdbId {
    /// Returns the raw numeric identifier.
    pub fn value(&self) -> u64 {
        match self {
            TmdbId::Movie(id) | TmdbId::Tv(id) => *id,
        }
    }

    /// Returns `true` when the identifier refers to a TV series.
    pub fn is_tv(&self) -> bool {
        matches!(self, TmdbId::Tv(_))
    }
}
