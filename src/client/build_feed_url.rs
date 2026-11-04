//! Builds the RSS feed URL of a Letterboxd member.

use crate::client::validate_username::validate_username;
use crate::result::Result;

pub use crate::client::letterboxd_base_url::LETTERBOXD_BASE_URL;

/// Builds the RSS feed URL for a member.
///
/// Letterboxd serves each member's feed at `https://letterboxd.com/<username>/rss/`.
///
/// # Errors
///
/// Returns [`LetterboxdError::InvalidUsername`](crate::error::LetterboxdError::InvalidUsername)
/// when the username is empty or contains characters that are not valid in a
/// Letterboxd username.
///
/// ```
/// # use letterboxd_rss_client_rs::client::build_feed_url;
/// assert_eq!(build_feed_url("jsiewert").unwrap(), "https://letterboxd.com/jsiewert/rss/");
/// ```
pub fn build_feed_url(username: &str) -> Result<String> {
    let username = validate_username(username)?;
    Ok(format!("{LETTERBOXD_BASE_URL}/{username}/rss/"))
}
