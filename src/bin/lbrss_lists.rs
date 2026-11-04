//! `lbrss-lists` — browse the curated lists in a Letterboxd member's RSS feed.
//!
//! Prints each curated list with its description and the films that were
//! inlined into the feed.
//!
//! ```text
//! lbrss-lists [--limit <N>] [--films <N>] [--json] <username>
//! ```

use std::time::Duration;

use clap::Parser;
use letterboxd_rss_client_rs::{LetterboxdClient, LetterboxdFeed};

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

/// Returns at most `limit` items, or all of them when `limit` is zero.
fn cap<T: Clone>(items: &[T], limit: usize) -> Vec<T> {
    if limit == 0 {
        items.to_vec()
    } else {
        items.iter().take(limit).cloned().collect()
    }
}

fn print_lists(feed: &LetterboxdFeed, list_limit: usize, film_limit: usize) {
    let lists = feed.lists();
    let shown = cap(&lists, list_limit);

    println!("Member:   {}", feed.member_name());
    println!(
        "Username: {}",
        feed.member_username().unwrap_or("(unknown)")
    );
    println!("Lists:    {} ({} shown)", lists.len(), shown.len());
    println!();

    if shown.is_empty() {
        println!("(no curated lists in this feed)");
        return;
    }

    for list in shown {
        let total = list
            .total_count()
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("{}", list.title);
        println!("  Link:        {}", list.link);
        if let Some(description) = list.description_text.as_deref() {
            let description = truncate(description, 200);
            if !description.is_empty() {
                println!("  Description: {description}");
            }
        }
        println!("  Films:       {total} total");
        println!();

        let films = cap(&list.films, film_limit);
        for film in &films {
            println!("  {:>3}. {}", film.position, film.title);
        }
        if list.is_truncated() && film_limit != 0 && list.films.len() > film_limit {
            println!(
                "  (showing {} of {} inlined films)",
                film_limit,
                list.films.len()
            );
        }
        println!();
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "lbrss-lists",
    version,
    about = "List the curated lists of a Letterboxd member from their RSS feed"
)]
struct Args {
    /// Letterboxd username whose feed should be fetched.
    username: String,

    /// Maximum number of lists to show (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    limit: usize,

    /// Maximum number of films to show per list (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    films: usize,

    /// Output the lists as pretty JSON.
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
        let lists = cap(&feed.lists(), args.limit);
        match serde_json::to_string_pretty(&lists) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    print_lists(&feed, args.limit, args.films);
}
