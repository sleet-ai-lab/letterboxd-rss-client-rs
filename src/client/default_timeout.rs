//! The default request timeout of the Letterboxd RSS client.

use std::time::Duration;

/// The default request timeout of the client.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
