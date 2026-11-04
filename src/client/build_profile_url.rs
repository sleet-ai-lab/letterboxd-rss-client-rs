//! Builds the profile URL of a Letterboxd member.

use crate::client::build_feed_url::LETTERBOXD_BASE_URL;
use crate::client::validate_username::validate_username;
use crate::result::Result;

/// Builds the profile URL for a member.
///
/// This is a convenience helper for linking to a member's page. The crate
/// itself only parses the RSS feed, and this URL is an ordinary HTML page,
/// not a feed.
///
/// ```
/// # use letterboxd_rss_client_rs::client::build_profile_url;
/// assert_eq!(build_profile_url("jsiewert").unwrap(), "https://letterboxd.com/jsiewert/");
/// ```
pub fn build_profile_url(username: &str) -> Result<String> {
    let username = validate_username(username)?;
    Ok(format!("{LETTERBOXD_BASE_URL}/{username}/"))
}
