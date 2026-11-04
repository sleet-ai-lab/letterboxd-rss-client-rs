//! Small helpers shared across the crate.

pub mod collapse_whitespace;
pub mod extract_poster_url;
pub mod html_to_text;

pub use collapse_whitespace::collapse_whitespace;
pub use extract_poster_url::extract_poster_url;
pub use html_to_text::html_to_text;
