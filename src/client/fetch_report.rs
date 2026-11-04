//! The outcome of fetching the feeds of several members.

use crate::error::LetterboxdError;
use crate::types::LetterboxdFeed;

/// The outcome of fetching the feeds of several members.
#[derive(Debug, Default)]
pub struct FetchReport {
    /// The feeds, in the same order as the requested usernames.
    pub feeds: Vec<LetterboxdFeed>,
    /// The members that could not be fetched, with the reason.
    pub errors: Vec<(String, LetterboxdError)>,
}

impl FetchReport {
    /// Returns the number of feeds that were fetched successfully.
    pub fn success_count(&self) -> usize {
        self.feeds.len() - self.errors.len()
    }

    /// Returns whether every member was fetched successfully.
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty()
    }
}
