//! The top-level RSS feed for a Letterboxd member.

use serde::{Deserialize, Serialize};

use crate::types::{FeedItem, FeedItemKind};

/// A parsed Letterboxd RSS feed.
///
/// Letterboxd serves one feed per member at
/// `https://letterboxd.com/<username>/rss/`. The feed mixes two kinds of
/// content — diary entries and curated lists — so [`LetterboxdFeed::items`]
/// holds every item while the helper methods filter by kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetterboxdFeed {
    /// The channel title, for example `Letterboxd - Josh Siewert`.
    pub title: String,
    /// The channel link, pointing at the member's profile.
    pub link: String,
    /// The channel description.
    pub description: Option<String>,
    /// The feed's self-referencing Atom link.
    pub atom_link: Option<String>,
    /// Every item in the feed, in document order.
    pub items: Vec<FeedItem>,
}

impl LetterboxdFeed {
    /// Returns an empty feed.
    ///
    /// This is used to keep the results of a multi-member fetch aligned with
    /// the requested usernames when one of them fails.
    pub fn empty() -> Self {
        Self {
            title: String::new(),
            link: String::new(),
            description: None,
            atom_link: None,
            items: Vec::new(),
        }
    }

    /// Returns only the diary entries from the feed.
    pub fn diary_entries(&self) -> Vec<&crate::types::DiaryEntry> {
        self.items
            .iter()
            .filter_map(|item| match item {
                FeedItem::Diary(entry) => Some(entry),
                _ => None,
            })
            .collect()
    }

    /// Returns only the curated lists from the feed.
    pub fn lists(&self) -> Vec<&crate::types::LetterboxdList> {
        self.items
            .iter()
            .filter_map(|item| match item {
                FeedItem::List(list) => Some(list),
                _ => None,
            })
            .collect()
    }

    /// Returns the number of items of the given kind.
    pub fn count_of_kind(&self, kind: FeedItemKind) -> usize {
        self.items.iter().filter(|item| item.kind() == kind).count()
    }

    /// Returns the member's display name, derived from the channel title.
    ///
    /// Letterboxd writes channel titles as `Letterboxd - <display name>`, so
    /// this strips that prefix. When the title does not have the expected
    /// shape the original title is returned unchanged.
    pub fn member_name(&self) -> &str {
        self.title
            .strip_prefix("Letterboxd - ")
            .unwrap_or(&self.title)
            .trim()
    }

    /// Returns the member's username, derived from the channel link.
    ///
    /// The channel link has the shape `https://letterboxd.com/<username>/`.
    pub fn member_username(&self) -> Option<&str> {
        let rest = self.link.split("letterboxd.com/").nth(1)?;
        let username = rest.split(['/', '?', '#']).next()?;
        if username.is_empty() {
            None
        } else {
            Some(username)
        }
    }
}
