//! A single entry in a Letterboxd RSS feed.

use serde::{Deserialize, Serialize};

use crate::types::{DiaryEntry, FeedItemKind, LetterboxdList, LikedReview};

/// A single item from a Letterboxd RSS feed.
///
/// Letterboxd multiplexes three different kinds of content into one feed:
/// diary entries, curated lists, and (for members who enable it) likes. The
/// concrete kind is recovered from the `guid` prefix, which Letterboxd writes
/// as `letterboxd-watch-<id>`, `letterboxd-review-<id>` or
/// `letterboxd-list-<id>`.
///
/// Use [`FeedItem::kind`] to branch on the variant without matching twice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FeedItem {
    /// A diary entry: a film the member watched, possibly with a review.
    Diary(DiaryEntry),
    /// A curated list of films.
    List(LetterboxdList),
    /// A review the member liked.
    LikedReview(LikedReview),
}

impl FeedItem {
    /// Returns the kind of this feed item.
    pub fn kind(&self) -> FeedItemKind {
        match self {
            FeedItem::Diary(_) => FeedItemKind::Diary,
            FeedItem::List(_) => FeedItemKind::List,
            FeedItem::LikedReview(_) => FeedItemKind::LikedReview,
        }
    }

    /// Returns the item's GUID exactly as Letterboxd wrote it.
    pub fn guid(&self) -> &str {
        match self {
            FeedItem::Diary(entry) => &entry.guid,
            FeedItem::List(list) => &list.guid,
            FeedItem::LikedReview(review) => &review.guid,
        }
    }

    /// Returns the item's `<title>` exactly as Letterboxd wrote it.
    pub fn title(&self) -> &str {
        match self {
            FeedItem::Diary(entry) => &entry.title,
            FeedItem::List(list) => &list.title,
            FeedItem::LikedReview(review) => &review.title,
        }
    }

    /// Returns the item's `<link>` exactly as Letterboxd wrote it.
    pub fn link(&self) -> &str {
        match self {
            FeedItem::Diary(entry) => &entry.link,
            FeedItem::List(list) => &list.link,
            FeedItem::LikedReview(review) => &review.link,
        }
    }
}
