//! `lbrss-diary` — browse the diary entries in a Letterboxd member's RSS feed.
//!
//! Prints each watched film with its rating, watched date, review and links,
//! with optional filtering by rating, year and presence of a review.
//!
//! ```text
//! lbrss-diary [--limit <N>] [--min-rating <R>] [--watched-year <YEAR>]
//!             [--with-reviews] [--json] <username>
//! ```

use std::time::Duration;

use clap::Parser;
use letterboxd_rss_client_rs::{DiaryEntry, LetterboxdClient, LetterboxdFeed};

/// Truncate `text` to at most `max` characters, appending an ellipsis.
fn truncate(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max).collect();
    out.push('…');
    out
}

/// Whether `entry` passes the requested filters.
fn matches_filters(
    entry: &DiaryEntry,
    min_rating: Option<f64>,
    watched_year: Option<i32>,
    with_reviews: bool,
) -> bool {
    if with_reviews && !entry.has_review() {
        return false;
    }
    if let Some(minimum) = min_rating {
        match entry.rating_value() {
            Some(value) if value >= minimum => {}
            _ => return false,
        }
    }
    if let Some(year) = watched_year {
        match entry.watched_date {
            Some(date) if date.year() == year => {}
            _ => return false,
        }
    }
    true
}

fn print_entry(entry: &DiaryEntry) {
    let year = entry
        .film_year
        .map(|year| year.to_string())
        .unwrap_or_else(|| "—".to_string());
    let rating = match entry.rating.as_ref() {
        Some(rating) => format!("{} ({})", rating.stars(), rating.value()),
        None => "unrated".to_string(),
    };
    let watched = entry
        .watched_date
        .map(|date| date.to_string())
        .unwrap_or_else(|| "—".to_string());

    println!("{}", entry.display_title());
    println!("  Year:    {year}");
    println!("  Rating:  {rating}");
    println!("  Watched: {watched}");
    println!(
        "  Rewatch: {}",
        if entry.is_rewatch() { "yes" } else { "no" }
    );
    println!(
        "  Liked:   {}",
        if entry.liked.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }
    );
    println!("  Link:    {}", entry.link);
    println!(
        "  Poster:  {}",
        entry.poster_url.as_deref().unwrap_or("(none)")
    );
    if entry.has_review() {
        let review = truncate(entry.review_text.as_deref().unwrap_or_default(), 400);
        println!("  Review:  {review}");
    }
    println!();
}

fn print_entries(entries: &[&DiaryEntry], limit: usize) {
    for entry in entries.iter().take(limit) {
        print_entry(entry);
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "lbrss-diary",
    version,
    about = "List the diary entries of a Letterboxd member from their RSS feed"
)]
struct Args {
    /// Letterboxd username whose feed should be fetched.
    username: String,

    /// Maximum number of entries to show.
    #[arg(long, default_value_t = 50)]
    limit: usize,

    /// Only show entries rated at least this value.
    #[arg(long, value_name = "R")]
    min_rating: Option<f64>,

    /// Only show entries watched in this year.
    #[arg(long, value_name = "YEAR")]
    watched_year: Option<i32>,

    /// Only show entries that have a review.
    #[arg(long)]
    with_reviews: bool,

    /// Output the filtered entries as pretty JSON.
    #[arg(long)]
    json: bool,

    /// HTTP timeout in seconds.
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Custom User-Agent header.
    #[arg(long)]
    user_agent: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut client = LetterboxdClient::new().with_timeout(Duration::from_secs(args.timeout));
    if let Some(user_agent) = args.user_agent.clone() {
        client = client.with_user_agent(user_agent);
    }

    let feed: LetterboxdFeed = match client.fetch_feed(&args.username).await {
        Ok(feed) => feed,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    };

    let filtered: Vec<&DiaryEntry> = feed
        .diary_entries()
        .into_iter()
        .filter(|entry| {
            matches_filters(entry, args.min_rating, args.watched_year, args.with_reviews)
        })
        .collect();

    if args.json {
        match serde_json::to_string_pretty(&filtered) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    println!("Member:   {}", feed.member_name());
    println!(
        "Username: {}",
        feed.member_username().unwrap_or("(unknown)")
    );
    println!(
        "Entries:  {} matching ({} shown)",
        filtered.len(),
        filtered.len().min(args.limit)
    );
    println!();

    if filtered.is_empty() {
        println!("(no diary entries matched)");
        return;
    }

    print_entries(&filtered, args.limit);
}
