//! The result alias used across the Letterboxd RSS client.

use crate::error::LetterboxdError;

/// A convenient result alias for this crate.
pub type Result<T> = std::result::Result<T, LetterboxdError>;
