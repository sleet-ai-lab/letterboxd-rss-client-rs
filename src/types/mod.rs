//! The typed data model of a Letterboxd RSS feed.

pub mod diary_entry;
pub mod feed_item;
pub mod feed_item_kind;
pub mod film_reference;
pub mod letterboxd_feed;
pub mod letterboxd_list;
pub mod liked_review;
pub mod list_film;
pub mod published_date;
pub mod rating;
pub mod tmdb_id;
pub mod watch_entry;
pub mod watched_date;

pub use diary_entry::DiaryEntry;
pub use feed_item::FeedItem;
pub use feed_item_kind::FeedItemKind;
pub use film_reference::FilmReference;
pub use letterboxd_feed::LetterboxdFeed;
pub use letterboxd_list::LetterboxdList;
pub use liked_review::LikedReview;
pub use list_film::ListFilm;
pub use published_date::PublishedDate;
pub use rating::Rating;
pub use tmdb_id::TmdbId;
pub use watch_entry::WatchEntry;
pub use watched_date::WatchedDate;
