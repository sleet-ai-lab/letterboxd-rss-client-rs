# letterboxd-rss-client-rs

A fully typed Rust client for Letterboxd's public per-member RSS feeds. Letterboxd publishes a feed for every member at `https://letterboxd.com/<username>/rss/`, and that feed multiplexes three different kinds of content — diary entries, curated lists and liked reviews — into a single RSS 2.0 channel, with the payload of each item buried in CDATA-wrapped HTML. This crate fetches that feed over HTTP, parses the XML, and exposes it as an owned, strongly typed data model with no borrowed lifetimes, no `String` soup and no manual HTML scraping. Every type in the model derives `Serialize` and `Deserialize`, so a parsed feed can be written straight to JSON, and the crate ships four CLI binaries for inspecting a member's activity from the shell.

## Features

- **Typed data model.** `LetterboxdFeed`, `DiaryEntry`, `LetterboxdList`, `LikedReview`, `Rating`, `WatchedDate` and `TmdbId` are all concrete Rust types with documented fields.
- **Async client.** `LetterboxdClient` wraps `reqwest` with rustls, connection pooling, a configurable timeout and a descriptive default `User-Agent`.
- **Three item kinds.** Diary entries, curated lists and liked reviews are recovered from the feed and distinguished by their GUID prefix.
- **Half-star ratings.** Ratings are parsed into a `Rating` that is validated to lie on Letterboxd's `0.5..=5.0` half-star scale and can be rendered as star glyphs.
- **Watch dates.** `letterboxd:watchedDate` is parsed into a `WatchedDate` wrapping a `time::Date`, with year/month/day accessors.
- **Poster URLs.** The poster `<img>` is extracted out of the CDATA description body and exposed as `poster_url`.
- **Plain-text review extraction.** Review HTML is converted to readable plain text with the poster image and Letterboxd's automatic "Watched on …" notice removed.
- **List film extraction.** The films inlined in a list description are extracted in order, together with the number of films the feed truncated.
- **JSON-serializable types.** Every model type derives `serde::Serialize` and `serde::Deserialize`.
- **Four CLI binaries.** `lbrss`, `lbrss-summary`, `lbrss-lists` and `lbrss-diary` cover overview, statistics, lists and diary entries.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
letterboxd-rss-client-rs = "0.1"
```

## Usage

```rust
use letterboxd_rss_client_rs::{LetterboxdClient, LetterboxdError};

#[tokio::main]
async fn main() -> Result<(), LetterboxdError> {
    let client = LetterboxdClient::new();
    let feed = client.fetch_feed("jsiewert").await?;

    println!("{} ({})", feed.member_name(), feed.member_username().unwrap_or("?"));

    for entry in feed.diary_entries() {
        let stars = entry
            .rating
            .as_ref()
            .map(|rating| rating.stars())
            .unwrap_or_else(|| "unrated".to_string());

        println!(
            "{} ({})  {}  watched {}",
            entry.display_title(),
            entry.film_year.unwrap_or_default(),
            stars,
            entry.watched_date.map(|date| date.to_string()).unwrap_or_default(),
        );
    }

    Ok(())
}
```

The client is cheap to clone and pools connections, so create one and reuse it:

```rust
use std::time::Duration;
use letterboxd_rss_client_rs::LetterboxdClient;

let client = LetterboxdClient::new()
    .with_timeout(Duration::from_secs(10))
    .with_user_agent("my-tool/1.0");
```

Feeds can also be parsed straight from XML that you already have:

```rust
use letterboxd_rss_client_rs::parse_feed;

let xml = std::fs::read_to_string("feed.xml")?;
let feed = parse_feed(&xml)?;
println!("{} items", feed.items.len());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Data model

`LetterboxdFeed` holds the channel metadata and every item in document order:

| Field | Type | Description |
|---|---|---|
| `title` | `String` | The channel title, for example `Letterboxd - Josh Siewert`. |
| `link` | `String` | The channel link, pointing at the member's profile. |
| `description` | `Option<String>` | The channel description. |
| `atom_link` | `Option<String>` | The feed's self-referencing Atom link. |
| `items` | `Vec<FeedItem>` | Every item in the feed, in document order. |

The feed offers helper methods that filter by kind: `diary_entries()`, `lists()`, `liked_reviews()` and `count_of_kind(FeedItemKind)`. It also derives the member's identity from the channel metadata: `member_name()` strips the `Letterboxd - ` prefix from the title, and `member_username()` extracts the username from the link.

`FeedItem` is an enum with three variants, and the concrete kind is recovered from the GUID prefix:

| Variant | GUID prefix | Description |
|---|---|---|
| `FeedItem::Diary` | `letterboxd-watch-`, `letterboxd-review-` | A film the member watched, possibly with a review. |
| `FeedItem::List` | `letterboxd-list-` | A curated list of films. |
| `FeedItem::LikedReview` | `letterboxd-like-` | A review by another member that the feed owner liked. |

Every variant exposes `guid()`, `title()` and `link()` accessors that return the raw text exactly as Letterboxd wrote it.

### DiaryEntry

| Field | Type | Description |
|---|---|---|
| `guid` | `String` | The GUID, of the form `letterboxd-watch-<id>` or `letterboxd-review-<id>`. |
| `title` | `String` | The raw `<title>` text, for example `Split, 2016`. |
| `link` | `String` | The raw `<link>` URL pointing at the member's film page. |
| `pub_date` | `Option<String>` | The raw `<pubDate>` string. |
| `author` | `Option<String>` | The member's display name. |
| `film_title` | `Option<String>` | The film's title, from `letterboxd:filmTitle`. |
| `film_year` | `Option<i32>` | The film's release year, from `letterboxd:filmYear`. |
| `rating` | `Option<Rating>` | The rating the member gave, from `letterboxd:memberRating`. |
| `liked` | `Option<bool>` | Whether the member liked the film, from `letterboxd:memberLike`. |
| `rewatch` | `Option<bool>` | Whether this was a rewatch, from `letterboxd:rewatch`. |
| `watched_date` | `Option<WatchedDate>` | The date the film was watched, from `letterboxd:watchedDate`. |
| `tmdb_id` | `Option<TmdbId>` | The film's TMDB identifier, from `tmdb:movieId` or `tmdb:tvId`. |
| `poster_url` | `Option<String>` | The poster image URL, extracted from the description HTML. |
| `review_text` | `Option<String>` | The review body as plain text, when the member wrote one. |
| `review_html` | `Option<String>` | The review body as raw HTML, when the member wrote one. |

Convenience methods: `has_review()`, `is_rewatch()`, `display_title()` (falls back to the raw title when `film_title` is absent) and `rating_value()`.

### LetterboxdList

| Field | Type | Description |
|---|---|---|
| `guid` | `String` | The GUID, of the form `letterboxd-list-<id>`. |
| `title` | `String` | The raw `<title>` text. |
| `link` | `String` | The raw `<link>` URL pointing at the list page. |
| `pub_date` | `Option<String>` | The raw `<pubDate>` string, which is when the list was last updated. |
| `author` | `Option<String>` | The member's display name. |
| `description_text` | `Option<String>` | The list's description as plain text, without the film list or the truncation note. |
| `description_html` | `Option<String>` | The list's description as raw HTML. |
| `films` | `Vec<ListFilm>` | The films inlined in the feed description, in list order. |
| `truncated_count` | `Option<u64>` | The number of films omitted from the feed, when the list was truncated. |

Convenience methods: `visible_count()`, `total_count()` (visible films plus the truncated count) and `is_truncated()`. Each `ListFilm` carries a `title`, an optional `url` and a `position` starting at `1`.

### LikedReview

| Field | Type | Description |
|---|---|---|
| `guid` | `String` | The GUID, of the form `letterboxd-like-<id>`. |
| `title` | `String` | The raw `<title>` text. |
| `link` | `String` | The raw `<link>` URL. |
| `pub_date` | `Option<String>` | The raw `<pubDate>` string. |
| `author` | `Option<String>` | The review author's display name. |
| `film` | `Option<FilmReference>` | The film the review is about: `title`, `year`, `url` and `tmdb_id`. |
| `review_text` | `Option<String>` | The review body as plain text, when present. |
| `review_html` | `Option<String>` | The review body as raw HTML, when present. |

### Supporting types

| Type | Description |
|---|---|
| `Rating` | A validated half-star rating in `0.5..=5.0`. Exposes `value()`, `full_stars()`, `half_star()` and `stars()`, which renders `★★★★½` for 4.5. |
| `WatchedDate` | An ISO `YYYY-MM-DD` calendar date. Exposes `as_date()`, `year()`, `month()`, `day()` and `to_iso_string()`. |
| `PublishedDate` | An RFC 2822 `pubDate` timestamp with the original UTC offset preserved. |
| `TmdbId` | Either `TmdbId::Movie(u64)` or `TmdbId::Tv(u64)`. Exposes `value()` and `is_tv()`. |
| `LetterboxdError` | The error type of the crate. `is_member_not_found()` reports both the `MemberNotFound` variant and a `404` status error. |

## CLI

The crate installs four binaries. All of them take the username as a positional argument and accept `--timeout <SECONDS>` (default `30`) and `--user-agent <STRING>`.

### `lbrss` — overview

Prints the member, the item counts and a sample of each section.

| Flag | Default | Description |
|---|---|---|
| `--limit <LIMIT>` | `20` | Maximum number of items to show per section. |
| `--reviews` | off | Also print the review text for diary entries that have one. |
| `--json` | off | Output the raw feed as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ lbrss --limit 5 --reviews jsiewert
Member:   Josh Siewert
Username: jsiewert
Feed:     https://letterboxd.com/jsiewert/

Counts:
  Total items:   100
  Diary entries: 50
  Lists:         50
  Liked reviews: 0

Diary entries (5 shown):
  • Primetime (2026)  ★★★★½  watched 2026-11-03
      review: Finally saw this and Wow!
Merritt Weber is so so good in this. She was killing me. The whole cast ruled. Pattinson was perfect. This is Oppenheim’s year!
  • Women Talking (2022)  ★★★★★  watched 2026-11-03
      review: "Surely there must be something worth living for in this life, not only in the next."
1000/1000
  • Reds (1981)  ★★★★½  watched 2026-11-03
      review: Starring Diane Keaton and a bunch of journalists and communists who are asking eachother every 5 minutes if they want more coffee!!! OF COURSE I LIKED IT!
A true feat of epic filmmaking.
(I love how Nolan mirrors the interview style here for Interstallar. You can actually see some of Oppenheimer in …
  • Mobile Suit Gundam Hathaway (2021)  ★★★½  watched 2026-11-02
      review: Sweet animation.
  • Beavers (1988)  ★★★★½  watched 2026-11-02
      review: Trying to work through the rest of the ENTIRELY SHOT IN IMAX pantheon post The Odyssey.
This was fantastic!! Shot in Kananaskis, AB which is so cool. So fun learning how Canadian of a thing IMAX is.
Beavers are a wonder.

Lists (5 shown):
  • 2026 (10 films visible, 35 total)
      https://letterboxd.com/jsiewert/list/2026/
  • The Ten Greatest Films Ever Made (10 films visible, unknown total)
      https://letterboxd.com/jsiewert/list/the-ten-greatest-films-ever-made/
  • 1981 Top Ten (10 films visible, unknown total)
      https://letterboxd.com/jsiewert/list/1981-top-ten/
  • 1988 Top Ten (10 films visible, unknown total)
      https://letterboxd.com/jsiewert/list/1988-top-ten/
  • Canada (10 films visible, 58 total)
      https://letterboxd.com/jsiewert/list/canada/

Liked reviews (0 shown):
  (none)
```

### `lbrss-summary` — statistics

Prints aggregate counts, rating statistics, most-watched films, a per-year breakdown and the curated lists.

| Flag | Default | Description |
|---|---|---|
| `--json` | off | Output the summary as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ lbrss-summary mbill
Member:   MBill
Username: mbill
Feed:     https://letterboxd.com/mbill/

Items:
  Total:          51
  Diary entries:  50
  Lists:          1
  Liked reviews:  0

Diary statistics:
  Films watched:  50
  Rated:          0
  With reviews:   32
  Liked:          18
  Rewatches:      1
  Average rating: n/a
  Rating range:   n/a

Most watched films:
     1x  24 Hour Party People
     1x  All That Jazz
     1x  American Dream
     1x  Being There
     1x  Blade
     1x  Clayface
     1x  Coyote vs. Acme
     1x  Demon Seed
     1x  Edge of Tomorrow
     1x  Elsinore

Entries per year:
  2026: 50

Curated lists:
  • Movies I've ran on 35 mm at TIFF Lightbox and the Revue (102 films)
```

### `lbrss-lists` — curated lists

Prints each curated list with its description and the films that were inlined into the feed.

| Flag | Default | Description |
|---|---|---|
| `--limit <LIMIT>` | `0` | Maximum number of lists to show (`0` = unlimited). |
| `--films <FILMS>` | `0` | Maximum number of films to show per list (`0` = unlimited). |
| `--json` | off | Output the lists as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ lbrss-lists --limit 2 --films 4 maesnotebook
Member:   maesnotebook
Username: maesnotebook
Lists:    7 (2 shown)

would sell my soul to star in
  Link:        https://letterboxd.com/maesnotebook/list/would-sell-my-soul-to-star-in/
  Description: female parts I simply could never refuse
  Films:       96 total

    1. Gone Girl
    2. La La Land
    3. Girl, Interrupted
    4. The Hunger Games
  (showing 4 of 10 inlined films)

holy grail
  Link:        https://letterboxd.com/maesnotebook/list/holy-grail/
  Description: the core queens
  Films:       97 total

    1. Black Swan
    2. The Notebook
    3. Kill Bill: Vol. 1
    4. La La Land
  (showing 4 of 10 inlined films)
```

### `lbrss-diary` — diary entries

Prints each watched film with its rating, watched date, review and links, with optional filtering.

| Flag | Default | Description |
|---|---|---|
| `--limit <LIMIT>` | `50` | Maximum number of entries to show. |
| `--min-rating <R>` | — | Only show entries rated at least this value. |
| `--watched-year <YEAR>` | — | Only show entries watched in this year. |
| `--with-reviews` | off | Only show entries that have a review. |
| `--json` | off | Output the filtered entries as pretty JSON. |
| `--timeout <TIMEOUT>` | `30` | HTTP timeout in seconds. |
| `--user-agent <USER_AGENT>` | — | Custom `User-Agent` header. |

```console
$ lbrss-diary --limit 3 --min-rating 4 --with-reviews karsten
Member:   Karsten
Username: karsten
Entries:  4 matching (3 shown)

The Odyssey
  Year:    2026
  Rating:  ★★★★ (4)
  Watched: 2026-07-15
  Rewatch: no
  Liked:   yes
  Link:    https://letterboxd.com/karsten/film/the-odyssey-2026/
  Poster:  https://a.ltrbxd.com/resized/film-poster/1/2/5/5/3/9/4/1255394-the-odyssey-2026-0-600-0-900-crop.jpg?v=1eed046d0c
  Review:  Really enjoyed it. Got more into it as it went along. Really liked this interpretation of the Trojan Horse. Happy to see Hamish Patel in such a prominent role. Didn't have the same highest as some other Nolan for me though. The feel of this is really similar to Oppenheimer. Personally I think Interstellar (emotional best) and Tenet (action best) are still the Nolan highs for me.

Galaxy Quest
  Year:    1999
  Rating:  ★★★★½ (4.5)
  Watched: 2026-05-06
  Rewatch: no
  Liked:   yes
  Link:    https://letterboxd.com/karsten/film/galaxy-quest/
  Poster:  https://a.ltrbxd.com/resized/film-poster/5/1/2/2/7/51227-galaxy-quest-0-600-0-900-crop.jpg?v=77947eabcc
  Review:  Holds up. 2nd best movie where Sigourney Weaver talks to a space computer.

28 Years Later
  Year:    2025
  Rating:  ★★★★½ (4.5)
  Watched: 2025-07-09
  Rewatch: no
  Liked:   yes
  Link:    https://letterboxd.com/karsten/film/28-years-later/
  Poster:  https://a.ltrbxd.com/resized/film-poster/9/9/2/7/8/6/992786-28-years-later-0-600-0-900-crop.jpg?v=014e09c066
  Review:  Some tonal whiplash and I wish it was just a fully standalone story within the world established by 28 Days Later but I really loved it. Enjoyed how all over the place it was stylistically and tonally. A lot of the same editing as the slasher portions of Sunshine (a positive).
```

All four binaries support `--json`, which emits the parsed model rather than the text view:

```console
$ lbrss-diary --limit 1 --json karsten
[
  {
    "guid": "letterboxd-watch-1566213236",
    "title": "You Can See Everything, 2026 - ★★★★★",
    "link": "https://letterboxd.com/karsten/film/you-can-see-everything/",
    "pub_date": "Sun, 1 Nov 2026 12:20:29 +1300",
    "author": "Karsten",
    "film_title": "You Can See Everything",
    "film_year": 2026,
    "rating": 5.0,
    "liked": true,
    "rewatch": false,
    "watched_date": "2026-10-31",
    "tmdb_id": 1765072,
    "poster_url": "https://a.ltrbxd.com/resized/film-poster/1/6/2/9/4/4/3/1629443-you-can-see-everything-0-600-0-900-crop.jpg?v=84f6724d74",
    "review_text": null,
    "review_html": null
  },
  ...
]
```

## The feed format

Letterboxd serves one RSS 2.0 channel per member. The channel header looks like this:

```xml
<?xml version='1.0' encoding='utf-8'?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:letterboxd="https://letterboxd.com" xmlns:tmdb="https://themoviedb.org">
	<channel>
		<title>Letterboxd - Nonresistant Really</title>
		<link>https://letterboxd.com/notreallyfilms/</link>
		<description>Letterboxd - Nonresistant Really</description>
		<atom:link rel="self" href="https://letterboxd.com/notreallyfilms/rss/" type="application/rss+xml"/>
```

Three namespace prefixes are in play:

| Prefix | Namespace | Used for |
|---|---|---|
| `letterboxd:` | `https://letterboxd.com` | `filmTitle`, `filmYear`, `memberRating`, `memberLike`, `rewatch`, `watchedDate`. |
| `tmdb:` | `https://themoviedb.org` | `movieId` or `tvId`, depending on whether the film is a movie or a series. |
| `dc:` | `http://purl.org/dc/elements/1.1/` | `creator`, which holds the member's display name. |

The three kinds of item are multiplexed into the same channel and told apart by the `<guid>` prefix:

| GUID prefix | Kind | Notes |
|---|---|---|
| `letterboxd-watch-` | Diary entry | A plain watch, with no review. |
| `letterboxd-review-` | Diary entry | A watch with a written review. |
| `letterboxd-list-` | Curated list | Carries the inlined film list. |
| `letterboxd-like-` | Liked review | A review by another member that the feed owner liked. |

A diary entry item looks like this:

```xml
<item> <title>Split, 2016</title> <link>https://letterboxd.com/notreallyfilms/film/split-2016/</link> <guid isPermaLink="false">letterboxd-watch-1567593051</guid> <pubDate>Mon, 2 Nov 2026 07:37:59 +1300</pubDate> <letterboxd:watchedDate>2026-11-01</letterboxd:watchedDate> <letterboxd:rewatch>No</letterboxd:rewatch> <letterboxd:filmTitle>Split</letterboxd:filmTitle> <letterboxd:filmYear>2016</letterboxd:filmYear> <letterboxd:memberLike>No</letterboxd:memberLike> <tmdb:movieId>381288</tmdb:movieId> <description><![CDATA[ <p><img src="https://a.ltrbxd.com/resized/sm/upload/5e/pw/vs/km/8CHJcVc5IGXS1sC5DyjeMwTDEQH-0-600-0-900-crop.jpg?v=ab64d75800"/></p> <p>Watched on Sunday November 1, 2026.</p> ]]></description> <dc:creator>Nonresistant Really</dc:creator> </item>
```

Points worth knowing about the format:

- **The `<description>` is CDATA-wrapped HTML.** The item body is not a text field but a small HTML document. The first `<p>` usually holds only the poster `<img src="...">`, which is a resized CDN copy of the TMDB poster. Everything after it is the review prose.
- **A plain watch has no review.** Its description says only `Watched on Sunday November 1, 2026.`, which the parser recognises as an automatic notice and does not report as a review. Only `letterboxd-review-` items carry real prose, and only they get a `review_text`.
- **Lists are truncated.** Letterboxd inlines at most ten films into the list description and appends a note of the form `...plus 86 more. View the full list on Letterboxd`. The parser reads that marker into `truncated_count`, so `total_count()` returns the full size of the list while `films` holds only the visible ones.
- **There is no pagination.** The feed carries roughly the 50 most recent diary entries plus recently updated lists, and nothing more. Across the 16 captured feeds in `tests/fixtures/` no feed contains more than 50 diary entries, and the number of lists ranges from 0 to 50. A full diary therefore has to be scraped from the member's profile pages; the RSS feed is a rolling window.
- **The member's display name lives in the title.** The channel title is always `Letterboxd - <display name>`, which is why `member_name()` strips the prefix rather than using the title verbatim.

## Project layout

The library follows a strict one-item-per-file convention: one type or one function per file, with a `mod.rs` that only re-exports. This keeps each file small and makes the public surface easy to read from the directory listing.

```text
src/
├── lib.rs                        # crate root and public re-exports
├── error.rs                      # LetterboxdError
├── result.rs                     # Result<T>
├── bin/
│   ├── lbrss.rs                  # overview binary
│   ├── lbrss_diary.rs            # diary binary
│   ├── lbrss_lists.rs            # lists binary
│   └── lbrss_summary.rs          # summary binary
├── client/
│   ├── mod.rs
│   ├── build_feed_url.rs         # build_feed_url
│   ├── build_profile_url.rs      # build_profile_url
│   ├── default_timeout.rs        # DEFAULT_TIMEOUT
│   ├── fetch_report.rs           # FetchReport
│   ├── letterboxd_base_url.rs    # LETTERBOXD_BASE_URL
│   ├── letterboxd_client.rs      # LetterboxdClient
│   └── validate_username.rs      # validate_username
├── parse/
│   ├── mod.rs
│   ├── parse_feed.rs             # parse_feed
│   └── parse_item.rs             # parse_item
├── types/
│   ├── mod.rs
│   ├── diary_entry.rs            # DiaryEntry
│   ├── feed_item.rs              # FeedItem
│   ├── feed_item_kind.rs         # FeedItemKind
│   ├── film_reference.rs         # FilmReference
│   ├── letterboxd_feed.rs        # LetterboxdFeed
│   ├── letterboxd_list.rs        # LetterboxdList
│   ├── liked_review.rs           # LikedReview
│   ├── list_film.rs              # ListFilm
│   ├── published_date.rs         # PublishedDate
│   ├── rating.rs                 # Rating
│   ├── tmdb_id.rs                # TmdbId
│   ├── watch_entry.rs            # WatchEntry
│   └── watched_date.rs           # WatchedDate
└── util/
    ├── mod.rs
    ├── collapse_whitespace.rs    # collapse_whitespace
    ├── extract_poster_url.rs     # extract_poster_url
    └── html_to_text.rs           # html_to_text
```

The `tests/` directory holds two integration test crates and the captured feeds:

```text
tests/
├── fixtures/                     # 16 real captured Letterboxd feeds
│   ├── ackanuba.xml
│   ├── davey.xml
│   ├── davidehrlich.xml
│   ├── hponett.xml
│   ├── jaepeg.xml
│   ├── jsiewert.xml
│   ├── karsten.xml
│   ├── kurstboy.xml
│   ├── maesnotebook.xml
│   ├── mbill.xml
│   ├── mulch.xml
│   ├── nerd21.xml
│   ├── notreallyfilms.xml
│   ├── ruthel777.xml
│   ├── samholladay.xml
│   └── sarahnyc.xml
├── live_api.rs                   # live tests, enabled with LETTERBOXD_LIVE_TESTS=1
└── parse_fixtures.rs             # parses every fixture
```

The fixture test parses all 16 captures and checks the invariants that hold across real feeds:

```console
$ cargo test --test parse_fixtures -- --nocapture
parsed 16 fixtures: 960 items (694 diary, 266 lists, 2413 list films, 0 liked reviews)
  rated: 556, with review: 468, rewatches: 71, truncated lists: 161
```

The live test is skipped by default so the suite does not depend on the network. Enable it with:

```console
$ LETTERBOXD_LIVE_TESTS=1 cargo test --test live_api -- --nocapture
jsiewert: 100 items, 50 diary entries, 49 rated, member "Josh Siewert"
missing member reported as: unexpected http status 404 for https://letterboxd.com/zzz-not-a-real-letterboxd-user-9x8y7z/rss/
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
