//! Error types for the Letterboxd RSS client.
//!
//! Every fallible operation in this crate returns [`LetterboxdError`].

/// The error type returned by all fallible operations in this crate.
#[derive(Debug, thiserror::Error)]
pub enum LetterboxdError {
    /// The HTTP request to Letterboxd failed.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The HTTP response status was not a success status.
    #[error("unexpected http status {status} for {url}")]
    HttpStatus {
        /// The HTTP status code returned by Letterboxd.
        status: u16,
        /// The URL that was requested.
        url: String,
    },

    /// The response body could not be decoded as UTF-8 text.
    #[error("response body was not valid utf-8")]
    InvalidUtf8,

    /// The XML document could not be parsed.
    #[error("failed to parse xml: {0}")]
    Xml(#[from] quick_xml::Error),

    /// The XML document did not have the expected Letterboxd RSS shape.
    #[error("malformed letterboxd feed: {0}")]
    MalformedFeed(String),

    /// The requested member could not be found.
    #[error("letterboxd member not found: {0}")]
    MemberNotFound(String),

    /// The username was not in a usable form.
    #[error("invalid letterboxd username: {0}")]
    InvalidUsername(String),

    /// A date in the feed could not be parsed.
    #[error("failed to parse date `{value}` as {expected}")]
    InvalidDate {
        /// The raw date text that failed to parse.
        value: String,
        /// A human description of the expected format.
        expected: &'static str,
    },

    /// A rating value was outside the valid range.
    #[error("rating {0} is outside the range 0.5..=5.0")]
    InvalidRating(f64),
}

impl LetterboxdError {
    /// Returns `true` when the error indicates the member does not exist.
    ///
    /// A 404 from the RSS endpoint is how Letterboxd reports an unknown member,
    /// so this maps that case to [`LetterboxdError::MemberNotFound`].
    pub fn is_member_not_found(&self) -> bool {
        matches!(
            self,
            LetterboxdError::MemberNotFound(_) | LetterboxdError::HttpStatus { status: 404, .. }
        )
    }
}
