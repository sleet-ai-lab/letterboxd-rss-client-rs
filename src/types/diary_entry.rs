//! A diary entry: a film the member watched, possibly with a review.

use serde::{Deserialize, Serialize};

use crate::types::{Rating, TmdbId, WatchedDate};

/// A watched film recorded in a member's Letterboxd diary.
///
/// A diary entry is always present for both a plain watch (no review) and a
/// written review. Use [`DiaryEntry::has_review`] to tell them apart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiaryEntry {
    /// The GUID, of the form `letterboxd-watch-<id>` or
    /// `letterboxd-review-<id>`.
    pub guid: String,
    /// The raw `<title>` text, for example `Split, 2016`.
    pub title: String,
    /// The raw `<link>` URL pointing at the member's film page.
    pub link: String,
    /// The raw `<pubDate>` string.
    pub pub_date: Option<String>,
    /// The member's display name.
    pub author: Option<String>,
    /// The film's title, from `letterboxd:filmTitle`.
    pub film_title: Option<String>,
    /// The film's release year, from `letterboxd:filmYear`.
    pub film_year: Option<i32>,
    /// The rating the member gave, from `letterboxd:memberRating`.
    pub rating: Option<Rating>,
    /// Whether the member liked the film, from `letterboxd:memberLike`.
    pub liked: Option<bool>,
    /// Whether this was a rewatch, from `letterboxd:rewatch`.
    pub rewatch: Option<bool>,
    /// The date the film was watched, from `letterboxd:watchedDate`.
    pub watched_date: Option<WatchedDate>,
    /// The film's TMDB identifier, from `tmdb:movieId` or `tmdb:tvId`.
    pub tmdb_id: Option<TmdbId>,
    /// The poster image URL, extracted from the description HTML.
    pub poster_url: Option<String>,
    /// The review body as plain text, when the member wrote one.
    pub review_text: Option<String>,
    /// The review body as raw HTML, when the member wrote one.
    pub review_html: Option<String>,
}

impl DiaryEntry {
    /// Returns whether the entry carries a written review.
    pub fn has_review(&self) -> bool {
        self.review_text
            .as_ref()
            .map(|text| !text.trim().is_empty())
            .unwrap_or(false)
    }

    /// Returns whether the entry is a rewatch.
    pub fn is_rewatch(&self) -> bool {
        self.rewatch.unwrap_or(false)
    }

    /// Returns the film title, falling back to the raw title when the
    /// `letterboxd:filmTitle` element was absent.
    pub fn display_title(&self) -> &str {
        self.film_title.as_deref().unwrap_or(&self.title)
    }

    /// Returns the rating value, when the member rated the film.
    pub fn rating_value(&self) -> Option<f64> {
        self.rating.map(|rating| rating.value())
    }
}
