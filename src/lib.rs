//! A fully typed client for Letterboxd's public RSS feeds.
//!
//! Letterboxd publishes a per-member RSS feed at
//! `https://letterboxd.com/<username>/rss/`. The feed mixes three kinds of
//! content — diary entries, curated lists and liked reviews — and this crate
//! turns that XML into a typed, owned data model.
//!
//! # Example
//!
//! ```no_run
//! use letterboxd_rss_client_rs::{LetterboxdClient, LetterboxdError};
//!
//! # async fn run() -> Result<(), LetterboxdError> {
//! let client = LetterboxdClient::new();
//! let feed = client.fetch_feed("jsiewert").await?;
//!
//! for entry in feed.diary_entries() {
//!     println!(
//!         "{} ({}) - {:?}",
//!         entry.display_title(),
//!         entry.film_year.unwrap_or_default(),
//!         entry.rating_value()
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Layout
//!
//! The crate follows a strict one-item-per-file convention:
//!
//! * [`types`] holds the data model, one type per file.
//! * [`parse`] holds the XML parsers, one function per file.
//! * [`client`] holds the HTTP layer.
//! * [`util`] holds small shared helpers.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod client;
pub mod error;
pub mod parse;
pub mod result;
pub mod types;
pub mod util;

pub use client::{build_feed_url, build_profile_url, FetchReport, LetterboxdClient};
pub use error::LetterboxdError;
pub use parse::{parse_feed, parse_item};
pub use result::Result;
pub use types::{
    DiaryEntry, FeedItem, FeedItemKind, FilmReference, LetterboxdFeed, LetterboxdList, LikedReview,
    ListFilm, PublishedDate, Rating, TmdbId, WatchedDate,
};
pub use util::{collapse_whitespace, extract_poster_url, html_to_text};
