//! `lbrss-summary` — compact statistics for a Letterboxd member's RSS feed.
//!
//! Prints aggregate information about the feed — counts, rating statistics,
//! most-watched films, a per-year breakdown and the member's curated lists —
//! without listing every individual item.
//!
//! ```text
//! lbrss-summary [--json] <username>
//! ```

use std::collections::BTreeMap;
use std::time::Duration;

use clap::Parser;
use letterboxd_rss_client_rs::{LetterboxdClient, LetterboxdFeed};
use serde::Serialize;

/// A film title and how often it appears in the diary.
#[derive(Debug, Serialize)]
struct FilmCount {
    title: String,
    count: usize,
}

/// The number of diary entries watched in one year.
#[derive(Debug, Serialize)]
struct YearCount {
    year: i32,
    count: usize,
}

/// A curated list and its total film count.
#[derive(Debug, Serialize)]
struct ListSummary {
    title: String,
    link: String,
    total_films: Option<u64>,
}

/// The full statistics summary.
#[derive(Debug, Serialize)]
struct Summary {
    member_name: String,
    username: Option<String>,
    feed_link: String,
    total_items: usize,
    diary_entries: usize,
    lists: usize,
    rated_count: usize,
    with_review_count: usize,
    liked_count: usize,
    rewatch_count: usize,
    average_rating: Option<f64>,
    min_rating: Option<f64>,
    max_rating: Option<f64>,
    films_watched: usize,
    top_films: Vec<FilmCount>,
    years: Vec<YearCount>,
    curated_lists: Vec<ListSummary>,
}

fn build_summary(feed: &LetterboxdFeed) -> Summary {
    let entries = feed.diary_entries();

    let mut rated_count = 0usize;
    let mut with_review_count = 0usize;
    let mut liked_count = 0usize;
    let mut rewatch_count = 0usize;
    let mut rating_sum = 0.0f64;
    let mut min_rating: Option<f64> = None;
    let mut max_rating: Option<f64> = None;
    let mut title_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut year_counts: BTreeMap<i32, usize> = BTreeMap::new();

    for entry in &entries {
        if entry.has_review() {
            with_review_count += 1;
        }
        if entry.liked.unwrap_or(false) {
            liked_count += 1;
        }
        if entry.is_rewatch() {
            rewatch_count += 1;
        }

        if let Some(value) = entry.rating_value() {
            rated_count += 1;
            rating_sum += value;
            min_rating = Some(min_rating.map_or(value, |current| current.min(value)));
            max_rating = Some(max_rating.map_or(value, |current| current.max(value)));
        }

        *title_counts
            .entry(entry.display_title().to_string())
            .or_insert(0) += 1;

        if let Some(date) = entry.watched_date {
            *year_counts.entry(date.year()).or_insert(0) += 1;
        }
    }

    let average_rating = if rated_count == 0 {
        None
    } else {
        Some((rating_sum / rated_count as f64 * 100.0).round() / 100.0)
    };

    let mut top_films: Vec<FilmCount> = title_counts
        .into_iter()
        .map(|(title, count)| FilmCount { title, count })
        .collect();
    top_films.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.title.cmp(&b.title)));
    top_films.truncate(10);

    let years = year_counts
        .into_iter()
        .map(|(year, count)| YearCount { year, count })
        .collect();

    let curated_lists = feed
        .lists()
        .iter()
        .map(|list| ListSummary {
            title: list.title.clone(),
            link: list.link.clone(),
            total_films: list.total_count(),
        })
        .collect();

    Summary {
        member_name: feed.member_name().to_string(),
        username: feed.member_username().map(|name| name.to_string()),
        feed_link: feed.link.clone(),
        total_items: feed.items.len(),
        diary_entries: entries.len(),
        lists: feed.lists().len(),
        rated_count,
        with_review_count,
        liked_count,
        rewatch_count,
        average_rating,
        min_rating,
        max_rating,
        films_watched: entries.len(),
        top_films,
        years,
        curated_lists,
    }
}

fn print_summary(summary: &Summary) {
    println!("Member:   {}", summary.member_name);
    println!(
        "Username: {}",
        summary.username.as_deref().unwrap_or("(unknown)")
    );
    println!("Feed:     {}", summary.feed_link);
    println!();

    println!("Items:");
    println!("  Total:          {}", summary.total_items);
    println!("  Diary entries:  {}", summary.diary_entries);
    println!("  Lists:          {}", summary.lists);
    println!();

    println!("Diary statistics:");
    println!("  Films watched:  {}", summary.films_watched);
    println!("  Rated:          {}", summary.rated_count);
    println!("  With reviews:   {}", summary.with_review_count);
    println!("  Liked:          {}", summary.liked_count);
    println!("  Rewatches:      {}", summary.rewatch_count);
    match summary.average_rating {
        Some(average) => println!("  Average rating: {average:.2}"),
        None => println!("  Average rating: n/a"),
    }
    match (summary.min_rating, summary.max_rating) {
        (Some(min), Some(max)) => println!("  Rating range:   {min} – {max}"),
        _ => println!("  Rating range:   n/a"),
    }
    println!();

    println!("Most watched films:");
    if summary.top_films.is_empty() {
        println!("  (none)");
    }
    for film in &summary.top_films {
        println!("  {:>4}x  {}", film.count, film.title);
    }
    println!();

    println!("Entries per year:");
    if summary.years.is_empty() {
        println!("  (none)");
    }
    for year in &summary.years {
        println!("  {}: {}", year.year, year.count);
    }
    println!();

    println!("Curated lists:");
    if summary.curated_lists.is_empty() {
        println!("  (none)");
    }
    for list in &summary.curated_lists {
        let total = list
            .total_films
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("  • {} ({total} films)", list.title);
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "lbrss-summary",
    version,
    about = "Print statistics summary for a Letterboxd member's RSS feed"
)]
struct Args {
    /// Letterboxd username whose feed should be fetched.
    username: String,

    /// Output the summary as pretty JSON.
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

    let summary = build_summary(&feed);

    if args.json {
        match serde_json::to_string_pretty(&summary) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    print_summary(&summary);
}
