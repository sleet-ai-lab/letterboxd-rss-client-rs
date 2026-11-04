//! Integration test against the live Letterboxd API.
//!
//! The test is skipped unless `LETTERBOXD_LIVE_TESTS=1` is set, because it
//! talks to letterboxd.com and would otherwise make the test suite depend on
//! the network.
//!
//! ```text
//! LETTERBOXD_LIVE_TESTS=1 cargo test --test live_api -- --nocapture
//! ```

use std::time::Duration;

use letterboxd_rss_client_rs::{LetterboxdClient, LetterboxdError};

/// Returns `true` when the live tests are enabled.
fn live_tests_enabled() -> bool {
    std::env::var("LETTERBOXD_LIVE_TESTS").is_ok_and(|value| value == "1")
}

/// Builds a client with the timeout used by the live tests.
fn client() -> LetterboxdClient {
    LetterboxdClient::new().with_timeout(Duration::from_secs(30))
}

/// A known-good member: the feed must load and carry diary entries.
#[tokio::test]
async fn fetches_a_known_members_feed() {
    if !live_tests_enabled() {
        eprintln!("skipping: set LETTERBOXD_LIVE_TESTS=1 to run the live tests");
        return;
    }

    let feed = client()
        .fetch_feed("jsiewert")
        .await
        .expect("the feed of jsiewert must load");

    assert!(
        feed.items.len() > 50,
        "expected more than 50 items, got {}",
        feed.items.len()
    );

    assert_eq!(
        feed.member_name(),
        "Josh Siewert",
        "the channel title must yield the member's display name"
    );

    let rated = feed
        .diary_entries()
        .iter()
        .filter(|entry| entry.rating_value().is_some())
        .count();

    assert!(
        rated > 0,
        "expected at least one rated diary entry, got {rated}"
    );

    println!(
        "jsiewert: {} items, {} diary entries, {} rated, member {:?}",
        feed.items.len(),
        feed.diary_entries().len(),
        rated,
        feed.member_name()
    );
}

/// An unknown member must be reported as `is_member_not_found`.
#[tokio::test]
async fn reports_a_missing_member() {
    if !live_tests_enabled() {
        eprintln!("skipping: set LETTERBOXD_LIVE_TESTS=1 to run the live tests");
        return;
    }

    let error = client()
        .fetch_feed("zzz-not-a-real-letterboxd-user-9x8y7z")
        .await
        .expect_err("an unknown member must not produce a feed");

    assert!(
        error.is_member_not_found(),
        "expected a member-not-found error, got: {error}"
    );

    // Letterboxd reports an unknown member as a 404 on the RSS endpoint, and
    // the client maps that to the dedicated variant.
    assert!(
        matches!(error, LetterboxdError::MemberNotFound(_)),
        "expected the MemberNotFound variant, got: {error}"
    );

    println!("missing member reported as: {error}");
}
