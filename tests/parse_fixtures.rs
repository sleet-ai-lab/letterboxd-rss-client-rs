//! Integration test that parses every captured Letterboxd feed fixture.
//!
//! The fixtures in `tests/fixtures/` are real, unmodified captures of
//! `https://letterboxd.com/<username>/rss/`. They are used to prove that the
//! parser copes with the shapes Letterboxd actually emits: diary entries,
//! curated lists and liked reviews, with real ratings, watch dates, poster
//! images and truncated lists.

use std::fs;
use std::path::PathBuf;

use letterboxd_rss_client_rs::types::FeedItem;
use letterboxd_rss_client_rs::{parse_feed, LetterboxdFeed};

/// The directory holding the captured feeds.
const FIXTURE_DIR: &str = "tests/fixtures";

/// Counts the `<item>` elements in a raw feed document.
fn count_raw_items(xml: &str) -> usize {
    xml.matches("<item>").count()
}

/// Returns the fixture file names in a deterministic order.
fn fixture_paths() -> Vec<PathBuf> {
    let entries = fs::read_dir(FIXTURE_DIR)
        .unwrap_or_else(|error| panic!("cannot read {FIXTURE_DIR}: {error}"));

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xml"))
        .collect();

    paths.sort();
    assert!(!paths.is_empty(), "no xml fixtures found in {FIXTURE_DIR}");

    paths
}

/// Asserts the invariants that hold for every parsed feed.
fn assert_feed_invariants(name: &str, feed: &LetterboxdFeed, raw_items: usize) {
    assert!(
        !feed.title.is_empty(),
        "{name}: the channel title must not be empty"
    );
    assert!(
        feed.title.starts_with("Letterboxd - "),
        "{name}: the channel title must start with \"Letterboxd - \", got {:?}",
        feed.title
    );
    assert!(
        feed.link.contains("letterboxd.com/"),
        "{name}: the channel link must point at letterboxd.com, got {:?}",
        feed.link
    );

    let member_name = feed.member_name();
    assert!(
        !member_name.is_empty(),
        "{name}: the member name must not be empty"
    );
    assert_ne!(
        member_name,
        feed.title.as_str(),
        "{name}: the \"Letterboxd - \" prefix must have been stripped from the member name"
    );

    let username = feed
        .member_username()
        .unwrap_or_else(|| panic!("{name}: the member username must be present"));
    assert!(
        !username.is_empty(),
        "{name}: the member username must not be empty"
    );

    assert_eq!(
        feed.items.len(),
        raw_items,
        "{name}: the parsed item count must match the number of <item> elements"
    );

    for item in &feed.items {
        assert!(
            !item.guid().is_empty(),
            "{name}: every item must have a non-empty guid"
        );
        assert!(
            !item.title().is_empty(),
            "{name}: every item ({}) must have a non-empty title",
            item.guid()
        );
        assert!(
            item.link().starts_with("https://letterboxd.com/"),
            "{name}: the link of {} must start with https://letterboxd.com/, got {:?}",
            item.guid(),
            item.link()
        );
    }
}

/// Asserts the invariants that hold for every diary entry.
fn assert_diary_invariants(name: &str, entry: &letterboxd_rss_client_rs::DiaryEntry) {
    assert!(
        entry.film_title.is_some(),
        "{name}: the diary entry {} must have a film title",
        entry.guid
    );
    assert!(
        entry.film_year.is_some(),
        "{name}: the diary entry {} must have a film year",
        entry.guid
    );
    assert!(
        entry.watched_date.is_some(),
        "{name}: the diary entry {} must have a watched date",
        entry.guid
    );

    if let Some(rating) = entry.rating_value() {
        assert!(
            (0.5..=5.0).contains(&rating),
            "{name}: the rating {} of {} is outside 0.5..=5.0",
            rating,
            entry.guid
        );
    }
}

/// Asserts the invariants that hold for every curated list.
fn assert_list_invariants(name: &str, list: &letterboxd_rss_client_rs::LetterboxdList) {
    for (expected_position, film) in (1u32..).zip(list.films.iter()) {
        assert_eq!(
            film.position, expected_position,
            "{name}: the list {} must number its films consecutively from 1",
            list.guid
        );
        assert!(
            !film.title.trim().is_empty(),
            "{name}: the list {} has a film without a title",
            list.guid
        );
    }
}

/// Counts the `<item>` elements of every fixture and parses them all.
#[test]
fn every_fixture_parses_into_a_consistent_feed() {
    let paths = fixture_paths();

    let mut total_items = 0usize;
    let mut total_diary = 0usize;
    let mut total_lists = 0usize;
    let mut total_liked_reviews = 0usize;
    let mut total_rated = 0usize;
    let mut total_with_review = 0usize;
    let mut total_rewatch = 0usize;
    let mut total_truncated_lists = 0usize;
    let mut total_list_films = 0usize;

    let mut saw_review_text = false;
    let mut saw_five_star_rating = false;
    let mut saw_rewatch = false;
    let mut saw_truncated_count = false;
    let mut saw_poster_url = false;

    for path in &paths {
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        let xml = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        let raw_items = count_raw_items(&xml);
        assert!(
            raw_items > 0,
            "{name}: the fixture contains no <item> elements"
        );

        let feed = parse_feed(&xml)
            .unwrap_or_else(|error| panic!("{name}: the feed does not parse: {error}"));

        assert_feed_invariants(&name, &feed, raw_items);

        for item in &feed.items {
            match item {
                FeedItem::Diary(entry) => {
                    assert_diary_invariants(&name, entry);
                    total_diary += 1;
                    total_rated += usize::from(entry.rating_value().is_some());
                    total_with_review += usize::from(entry.has_review());
                    total_rewatch += usize::from(entry.is_rewatch());

                    if entry.has_review() {
                        saw_review_text = true;
                    }
                    if entry.rating_value() == Some(5.0) {
                        saw_five_star_rating = true;
                    }
                    if entry.is_rewatch() {
                        saw_rewatch = true;
                    }
                    if entry.poster_url.is_some() {
                        saw_poster_url = true;
                    }
                }
                FeedItem::List(list) => {
                    assert_list_invariants(&name, list);
                    total_lists += 1;
                    total_list_films += list.films.len();

                    if list.truncated_count.is_some() {
                        saw_truncated_count = true;
                        total_truncated_lists += 1;
                    }
                }
                FeedItem::LikedReview(_) => {
                    total_liked_reviews += 1;
                }
            }
        }

        total_items += feed.items.len();
    }

    // The fixtures must really contain data, otherwise the test would pass
    // vacuously.
    assert_eq!(
        paths.len(),
        16,
        "expected 16 fixtures in {FIXTURE_DIR}, found {}",
        paths.len()
    );
    assert!(
        total_items > 500,
        "expected more than 500 items across the fixtures, got {total_items}"
    );
    assert!(
        total_diary > 0,
        "expected the fixtures to contain diary entries"
    );
    assert!(
        total_lists > 0,
        "expected the fixtures to contain curated lists"
    );
    assert!(
        total_list_films > 0,
        "expected the fixtures to contain list films"
    );
    assert!(
        saw_review_text,
        "expected at least one diary entry with a review text"
    );
    assert!(
        saw_five_star_rating,
        "expected at least one diary entry rated exactly 5.0"
    );
    assert!(saw_rewatch, "expected at least one rewatch");
    assert!(
        saw_truncated_count,
        "expected at least one list with a truncated count"
    );
    assert!(
        saw_poster_url,
        "expected at least one diary entry with a poster url"
    );

    println!(
        "parsed {fixtures} fixtures: {items} items ({diary} diary, {lists} lists, {films} list films, {liked} liked reviews)",
        fixtures = paths.len(),
        items = total_items,
        diary = total_diary,
        lists = total_lists,
        films = total_list_films,
        liked = total_liked_reviews,
    );
    println!(
        "  rated: {rated}, with review: {with_review}, rewatches: {rewatch}, truncated lists: {truncated}",
        rated = total_rated,
        with_review = total_with_review,
        rewatch = total_rewatch,
        truncated = total_truncated_lists,
    );
}

/// The fixture file names are the usernames the feeds were captured from.
#[test]
fn fixture_names_are_letterboxd_usernames() {
    for path in fixture_paths() {
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();

        assert!(
            !stem.is_empty(),
            "the fixture {} has no name",
            path.display()
        );
        assert!(
            stem.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "the fixture name {stem} is not a plausible Letterboxd username"
        );
    }
}
