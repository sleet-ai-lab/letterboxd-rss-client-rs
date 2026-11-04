//! Validates a Letterboxd username.

use crate::error::LetterboxdError;
use crate::result::Result;

/// Validates a Letterboxd username and returns it unchanged.
///
/// # Errors
///
/// Returns [`LetterboxdError::InvalidUsername`] when the username is empty,
/// contains whitespace, or contains a path separator.
pub fn validate_username(username: &str) -> Result<&str> {
    let trimmed = username.trim();

    if trimmed.is_empty() {
        return Err(LetterboxdError::InvalidUsername(username.to_string()));
    }

    if trimmed != username {
        return Err(LetterboxdError::InvalidUsername(username.to_string()));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(LetterboxdError::InvalidUsername(username.to_string()));
    }

    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('?')
        || trimmed.contains('#')
    {
        return Err(LetterboxdError::InvalidUsername(username.to_string()));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(LetterboxdError::InvalidUsername(username.to_string()));
    }

    Ok(trimmed)
}
