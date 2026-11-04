//! `lbrss` — a general-purpose viewer for a Letterboxd member's RSS feed.
//!
//! Fetches the feed and prints a human-readable overview of the member,
//! their diary entries, their curated lists and the reviews they liked.
//!
//! ```text
//! lbrss [--limit <N>] [--reviews] [--json] <username>
//! ```

use std::time::Duration;

use clap::Parser;
use letterboxd_rss_client_rs::{DiaryEntry, LetterboxdClient, LetterboxdFeed, LetterboxdList};

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

fn print_diary_entry(entry: &DiaryEntry, show_reviews: bool) {
    let year = entry
        .film_year
        .map(|year| year.to_string())
        .unwrap_or_else(|| "—".to_string());
    let stars = entry
        .rating
        .as_ref()
        .map(|rating| rating.stars())
        .unwrap_or_else(|| "unrated".to_string());
    let watched = entry
        .watched_date
        .map(|date| date.to_string())
        .unwrap_or_else(|| "—".to_string());

    println!(
        "  • {} ({})  {}  watched {}",
        entry.display_title(),
        year,
        stars,
        watched
    );

    if show_reviews && entry.has_review() {
        let review = entry.review_text.as_deref().unwrap_or_default();
        println!("      review: {}", truncate(review, 300));
    }
}

fn print_list(list: &LetterboxdList) {
    let total = list
        .total_count()
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!(
        "  • {} ({} films visible, {} total)",
        list.title,
        list.visible_count(),
        total
    );
    println!("      {}", list.link);
}

fn print_overview(feed: &LetterboxdFeed, limit: usize, show_reviews: bool) {
    println!("Member:   {}", feed.member_name());
    println!(
        "Username: {}",
        feed.member_username().unwrap_or("(unknown)")
    );
    println!("Feed:     {}", feed.link);
    println!();

    println!("Counts:");
    println!("  Total items:   {}", feed.items.len());
    println!(
        "  Diary entries: {}",
        feed.count_of_kind(letterboxd_rss_client_rs::FeedItemKind::Diary)
    );
    println!(
        "  Lists:         {}",
        feed.count_of_kind(letterboxd_rss_client_rs::FeedItemKind::List)
    );
    println!();

    let entries = feed.diary_entries();
    println!("Diary entries ({} shown):", entries.len().min(limit));
    if entries.is_empty() {
        println!("  (none)");
    }
    for entry in entries.iter().take(limit) {
        print_diary_entry(entry, show_reviews);
    }
    println!();

    let lists = feed.lists();
    println!("Lists ({} shown):", lists.len().min(limit));
    if lists.is_empty() {
        println!("  (none)");
    }
    for list in lists.iter().take(limit) {
        print_list(list);
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "lbrss",
    version,
    about = "View a Letterboxd member's RSS feed: diary entries, lists and liked reviews"
)]
struct Args {
    /// Letterboxd username whose feed should be fetched.
    username: String,

    /// Maximum number of items to show per section.
    #[arg(long, default_value_t = 20)]
    limit: usize,

    /// Also print the review text for diary entries that have one.
    #[arg(long)]
    reviews: bool,

    /// Output the raw feed as pretty JSON.
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

    if args.json {
        match serde_json::to_string_pretty(&feed) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    print_overview(&feed, args.limit, args.reviews);
}
