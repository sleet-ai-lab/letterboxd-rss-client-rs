//! Parses a single `<item>` element of a Letterboxd RSS feed.

use std::collections::BTreeMap;

use quick_xml::events::{BytesRef, Event};
use quick_xml::Reader;

use crate::error::LetterboxdError;
use crate::result::Result;
use crate::types::{
    DiaryEntry, FeedItem, FilmReference, LetterboxdList, LikedReview, ListFilm, Rating, TmdbId,
    WatchedDate,
};
use crate::util::{collapse_whitespace, extract_poster_url, html_to_text};

/// Parses a raw `<item>` XML fragment.
///
/// The fragment is the XML of a single `<item>` element, exactly as Letterboxd
/// writes it: namespace-prefixed elements such as `letterboxd:filmTitle` and
/// `dc:creator`, and a `<description>` whose body is a CDATA section holding
/// HTML. The fragment may be the inner XML of the element or the element
/// itself, both are accepted.
///
/// The kind of item is recovered from the `<guid>` prefix:
///
/// | GUID prefix             | Variant                        |
/// |-------------------------|--------------------------------|
/// | `letterboxd-watch-`     | [`FeedItem::Diary`]            |
/// | `letterboxd-review-`    | [`FeedItem::Diary`]            |
/// | `letterboxd-list-`      | [`FeedItem::List`]             |
/// | `letterboxd-like-`      | [`FeedItem::LikedReview`]      |
///
/// # Errors
///
/// Returns [`LetterboxdError::Xml`] when the fragment is not well-formed XML.
///
/// ```
/// # use letterboxd_rss_client_rs::parse::parse_item::parse_item;
/// let raw = r#"<title>Split, 2016</title><link>https://letterboxd.com/x/film/split-2016/</link>
/// <guid isPermaLink="false">letterboxd-watch-1567593051</guid>"#;
/// let item = parse_item(raw).expect("valid item").expect("known guid");
/// assert_eq!(item.guid(), "letterboxd-watch-1567593051");
/// ```
pub fn parse_item(raw: &str) -> Result<Option<FeedItem>> {
    let fields = collect_fields(raw)?;

    let guid = match fields.get("guid") {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => return Ok(None),
    };

    let title = fields.get("title").cloned().unwrap_or_default();
    let link = fields.get("link").cloned().unwrap_or_default();
    let pub_date = fields.get("pubDate").cloned();
    let author = fields.get("creator").cloned();
    let description_html = fields.get("description").cloned();

    if guid.starts_with("letterboxd-watch-") || guid.starts_with("letterboxd-review-") {
        let entry = build_diary_entry(
            guid,
            title,
            link,
            pub_date,
            author,
            description_html,
            fields.get("memberRating"),
            fields.get("memberLike"),
            fields.get("rewatch"),
            fields.get("watchedDate"),
            fields.get("filmTitle"),
            fields.get("filmYear"),
            fields.get("movieId"),
            fields.get("tvId"),
        );
        return Ok(Some(FeedItem::Diary(entry)));
    }

    if guid.starts_with("letterboxd-list-") {
        let list = build_list(guid, title, link, pub_date, author, description_html);
        return Ok(Some(FeedItem::List(list)));
    }

    if guid.starts_with("letterboxd-like-") {
        let review = build_liked_review(
            guid,
            title,
            link,
            pub_date,
            author,
            description_html,
            fields.get("filmTitle"),
            fields.get("filmYear"),
            fields.get("movieId"),
            fields.get("tvId"),
        );
        return Ok(Some(FeedItem::LikedReview(review)));
    }

    Ok(None)
}

/// Collects the text of every leaf element of the fragment, keyed by the
/// element's local name with any namespace prefix removed.
///
/// The fragment may or may not still be wrapped in its `<item>` element; both
/// the inner XML and the whole element are accepted.
fn collect_fields(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut reader = Reader::from_str(raw);
    reader.config_mut().trim_text(true);

    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut text = String::new();
    let mut depth = 0usize;
    // The depth at which the leaf elements of the fragment live. A wrapping
    // `<item>` element pushes them one level deeper.
    let mut base: Option<usize> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let name = local_name(event.name().as_ref());
                if base.is_none() {
                    // The first element decides where the leaves live.
                    base = Some(if name == "item" { 1 } else { 0 });
                }
                if depth == base.unwrap_or(0) {
                    current = Some(name);
                    text.clear();
                }
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                if depth == base.unwrap_or(0) {
                    if let Some(key) = current.take() {
                        let value = decode_entities(text.trim());
                        fields.insert(key, value);
                    }
                    text.clear();
                }
            }
            Ok(Event::Empty(event)) => {
                if base.is_none() {
                    base = Some(0);
                }
                if depth == base.unwrap_or(0) {
                    let name = local_name(event.name().as_ref());
                    fields.insert(name, String::new());
                }
            }
            Ok(Event::CData(data)) => {
                if depth == base.unwrap_or(0) + 1 {
                    text.push_str(&String::from_utf8_lossy(data.as_ref()));
                }
            }
            Ok(Event::Text(value)) => {
                if depth == base.unwrap_or(0) + 1 {
                    let decoded = value.xml_content().unwrap_or_default().into_owned();
                    text.push_str(&decoded);
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                if depth == base.unwrap_or(0) + 1 {
                    text.push_str(&resolve_reference(&reference));
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(LetterboxdError::Xml(error)),
            _ => {}
        }
    }

    Ok(fields)
}

/// Builds a diary entry from the parsed fields of a watch or review item.
#[allow(clippy::too_many_arguments)]
fn build_diary_entry(
    guid: String,
    title: String,
    link: String,
    pub_date: Option<String>,
    author: Option<String>,
    description_html: Option<String>,
    rating_text: Option<&String>,
    like_text: Option<&String>,
    rewatch_text: Option<&String>,
    watched_date_text: Option<&String>,
    film_title: Option<&String>,
    film_year: Option<&String>,
    movie_id: Option<&String>,
    tv_id: Option<&String>,
) -> DiaryEntry {
    let is_review = guid.starts_with("letterboxd-review-");

    let poster_url = description_html.as_deref().and_then(extract_poster_url);

    let prose = description_html
        .as_deref()
        .map(|html| review_prose(html, is_review))
        .unwrap_or_default();

    let review_text = if prose.is_empty() { None } else { Some(prose) };

    // The raw description is kept as HTML whenever it carries review prose.
    let review_html = if review_text.is_some() {
        description_html.map(|html| html.trim().to_string())
    } else {
        None
    };

    DiaryEntry {
        guid,
        title,
        link,
        pub_date,
        author,
        film_title: film_title.map(|value| decode_entities(value.trim())),
        film_year: film_year.and_then(|value| value.trim().parse::<i32>().ok()),
        rating: rating_text.and_then(|value| Rating::new(parse_float(value)).ok()),
        liked: like_text.and_then(|value| parse_yes_no(value)),
        rewatch: rewatch_text.and_then(|value| parse_yes_no(value)),
        watched_date: watched_date_text.and_then(|value| WatchedDate::parse(value.trim()).ok()),
        tmdb_id: parse_tmdb_id(movie_id, tv_id),
        poster_url,
        review_text,
        review_html,
    }
}

/// Builds a curated list from the parsed fields of a list item.
fn build_list(
    guid: String,
    title: String,
    link: String,
    pub_date: Option<String>,
    author: Option<String>,
    description_html: Option<String>,
) -> LetterboxdList {
    let films = description_html
        .as_deref()
        .map(list_films)
        .unwrap_or_default();

    let truncated_count = description_html.as_deref().and_then(truncated_count);

    let description_text = description_html
        .as_deref()
        .map(list_description_text)
        .filter(|text| !text.is_empty());

    LetterboxdList {
        guid,
        title,
        link,
        pub_date,
        author,
        description_text,
        description_html: description_html.map(|html| html.trim().to_string()),
        films,
        truncated_count,
    }
}

/// Builds a liked review from the parsed fields of a like item.
#[allow(clippy::too_many_arguments)]
fn build_liked_review(
    guid: String,
    title: String,
    link: String,
    pub_date: Option<String>,
    author: Option<String>,
    description_html: Option<String>,
    film_title: Option<&String>,
    film_year: Option<&String>,
    movie_id: Option<&String>,
    tv_id: Option<&String>,
) -> LikedReview {
    let prose = description_html
        .as_deref()
        .map(|html| review_prose(html, true))
        .unwrap_or_default();

    let review_text = if prose.is_empty() { None } else { Some(prose) };

    // The raw description is kept as HTML whenever it carries review prose.
    let review_html = if review_text.is_some() {
        description_html.map(|html| html.trim().to_string())
    } else {
        None
    };

    let film = film_title.map(|value| FilmReference {
        title: decode_entities(value.trim()),
        year: film_year.and_then(|value| value.trim().parse::<i32>().ok()),
        url: Some(link.clone()).filter(|url| !url.is_empty()),
        tmdb_id: parse_tmdb_id(movie_id, tv_id),
    });

    LikedReview {
        guid,
        title,
        link,
        pub_date,
        author,
        film,
        review_text,
        review_html,
    }
}

/// Returns the review prose of a description, with the poster image removed.
///
/// A plain watch carries no review: its description only says "Watched on …",
/// which is not a review and is therefore dropped.
fn review_prose(html: &str, is_review: bool) -> String {
    let stripped = strip_poster(html);
    let text = collapse_whitespace(&html_to_text(&stripped));

    if text.is_empty() {
        return String::new();
    }

    if !is_review && is_watched_on_notice(&text) {
        return String::new();
    }

    text
}

/// Returns `true` when the prose is Letterboxd's automatic "Watched on …" note.
fn is_watched_on_notice(text: &str) -> bool {
    let lowered = text.trim().to_ascii_lowercase();
    if !lowered.starts_with("watched on ") {
        return false;
    }

    // The notice is a single sentence ending in a period.
    match lowered.find('.') {
        Some(end) => lowered[end + 1..].trim().is_empty(),
        None => true,
    }
}

/// Removes the poster image and any paragraph that only wraps one.
fn strip_poster(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find("<p") {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        let remainder = &after[end + 1..];

        let inner = if let Some(close) = remainder.find("</p>") {
            &remainder[..close]
        } else {
            remainder
        };

        if only_contains_image(inner) {
            let consumed = match remainder.find("</p>") {
                Some(close) => end + 1 + close + "</p>".len(),
                None => end + 1,
            };
            out.push_str(&rest[..start]);
            rest = &rest[start + consumed..];
            continue;
        }

        out.push_str(&rest[..start + end + 1]);
        rest = remainder;
    }

    out.push_str(rest);
    out
}

/// Returns `true` when the HTML fragment holds nothing but images and spaces.
fn only_contains_image(fragment: &str) -> bool {
    let lowered = fragment.to_ascii_lowercase();
    let mut rest = lowered.as_str();
    let mut saw_image = false;

    while let Some(start) = rest.find("<img") {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            return false;
        };
        saw_image = true;
        rest = &after[end + 1..];
    }

    if !saw_image {
        return false;
    }

    // Anything left over that looks like a tag means this is not just an
    // image; anything left over that is not whitespace means there is prose.
    !rest.contains('<') && rest.trim().is_empty()
}

/// Extracts the films listed in a list description, in order.
fn list_films(html: &str) -> Vec<ListFilm> {
    let mut films = Vec::new();
    let mut rest = html;

    while let Some(start) = rest.find("<li") {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        let remainder = &after[end + 1..];

        let Some(close) = remainder.find("</li>") else {
            break;
        };
        let inner = &remainder[..close];

        if let Some(film) = list_film(inner, films.len() as u32 + 1) {
            films.push(film);
        }

        rest = &remainder[close + "</li>".len()..];
    }

    films
}

/// Builds a [`ListFilm`] from the contents of an `<li>` element.
fn list_film(inner: &str, position: u32) -> Option<ListFilm> {
    let link_start = inner.find("<a")?;
    let link_after = &inner[link_start..];
    let link_end = link_after.find('>')?;
    let link_tag = &link_after[..=link_end];
    let link_rest = &link_after[link_end + 1..];

    let url = html_attribute(link_tag, "href");
    let title_end = link_rest.find("</a>")?;
    let title = decode_entities(link_rest[..title_end].trim());

    if title.is_empty() {
        return None;
    }

    Some(ListFilm {
        title,
        url,
        position,
    })
}

/// Reads an attribute value out of an HTML tag.
fn html_attribute(tag: &str, attribute: &str) -> Option<String> {
    let lowered = tag.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lowered[cursor..].find(attribute) {
        let position = cursor + offset;
        let preceded_by_space =
            position > 0 && lowered.as_bytes()[position - 1].is_ascii_whitespace();
        let followed_by_equals = lowered[position + attribute.len()..].starts_with('=');

        if preceded_by_space && followed_by_equals {
            let rest = &tag[position + attribute.len() + 1..];
            let trimmed = rest.trim_start();
            let quote = trimmed.chars().next()?;

            if quote == '"' || quote == '\'' {
                let body = &trimmed[1..];
                let end = body.find(quote)?;
                return Some(body[..end].to_string());
            }

            let end = trimmed
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(trimmed.len());
            return Some(trimmed[..end].to_string());
        }

        cursor = position + attribute.len();
    }

    None
}

/// Extracts the number of omitted films from a list description.
fn truncated_count(html: &str) -> Option<u64> {
    find_plus_more(html).map(|(_, count)| count)
}

/// Finds the "plus N more" note of a truncated list.
///
/// Returns the byte offset of the number and the number itself.
fn find_plus_more(html: &str) -> Option<(usize, u64)> {
    let lowered = html.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lowered[cursor..].find("plus ") {
        let position = cursor + offset;
        let digits_start = position + "plus ".len();
        let digits: String = lowered[digits_start..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        if digits.is_empty() {
            cursor = position + "plus ".len();
            continue;
        }

        let after = &lowered[digits_start + digits.len()..];
        let trimmed = after.trim_start();
        if trimmed.starts_with("more")
            && trimmed
                .chars()
                .nth(4)
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(true)
        {
            if let Ok(count) = digits.parse::<u64>() {
                return Some((digits_start, count));
            }
        }

        cursor = digits_start + digits.len();
    }

    None
}

/// Builds the plain-text description of a list, without the film list and
/// without the "…plus N more" trailer.
fn list_description_text(html: &str) -> String {
    let without_list = strip_list_elements(html);
    let without_trailer = strip_list_trailer(&without_list);

    collapse_whitespace(&html_to_text(&without_trailer))
}

/// Removes the `<ul>`/`<ol>` element holding the films of a list.
fn strip_list_elements(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    loop {
        let ul = rest.find("<ul");
        let ol = rest.find("<ol");

        let (start, close_tag) = match (ul, ol) {
            (Some(ul), Some(ol)) if ol < ul => (ol, "</ol>"),
            (Some(ul), _) => (ul, "</ul>"),
            (None, Some(ol)) => (ol, "</ol>"),
            (None, None) => break,
        };

        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        let remainder = &after[end + 1..];
        let close = match remainder.find(close_tag) {
            Some(position) => position,
            None => remainder.len(),
        };

        out.push_str(&rest[..start]);
        rest = &remainder[(close + close_tag.len()).min(remainder.len())..];
    }

    out.push_str(rest);
    out
}

/// Removes the trailing "…plus N more. View the full list on Letterboxd." note.
fn strip_list_trailer(html: &str) -> String {
    let Some((position, _)) = find_plus_more(html) else {
        return html.to_string();
    };

    // Walk back to the start of the paragraph holding the note.
    let head = &html[..position];
    let paragraph_start = head.rfind("<p").unwrap_or_default();

    html[..paragraph_start].to_string()
}

/// Parses a `Yes`/`No` flag.
fn parse_yes_no(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Parses a float, ignoring anything that is not numeric.
fn parse_float(value: &str) -> f64 {
    value.trim().parse::<f64>().unwrap_or(f64::NAN)
}

/// Builds the TMDB identifier from the `tmdb:movieId` / `tmdb:tvId` elements.
fn parse_tmdb_id(movie_id: Option<&String>, tv_id: Option<&String>) -> Option<TmdbId> {
    if let Some(value) = movie_id.and_then(|value| value.trim().parse::<u64>().ok()) {
        return Some(TmdbId::Movie(value));
    }
    if let Some(value) = tv_id.and_then(|value| value.trim().parse::<u64>().ok()) {
        return Some(TmdbId::Tv(value));
    }
    None
}

/// Decodes HTML entities in a text field.
fn decode_entities(text: &str) -> String {
    html_escape::decode_html_entities(text).into_owned()
}

/// Resolves an XML entity or character reference such as `&#039;` or `&amp;`.
///
/// `quick_xml` reports references as their own event instead of folding them
/// into the surrounding text, so they have to be resolved by hand.
fn resolve_reference(reference: &BytesRef<'_>) -> String {
    if let Ok(Some(ch)) = reference.resolve_char_ref() {
        return ch.to_string();
    }

    let raw = reference.decode().unwrap_or_default().into_owned();
    decode_entities(&raw)
}

/// Strips a namespace prefix from an element name.
fn local_name(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    match text.split_once(':') {
        Some((_, local)) => local.to_string(),
        None => text.into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATCH_ITEM: &str = r#" <title>Split, 2016</title> <link>https://letterboxd.com/notreallyfilms/film/split-2016/</link> <guid isPermaLink="false">letterboxd-watch-1567593051</guid> <pubDate>Mon, 2 Nov 2026 07:37:59 +1300</pubDate> <letterboxd:watchedDate>2026-11-01</letterboxd:watchedDate> <letterboxd:rewatch>No</letterboxd:rewatch> <letterboxd:filmTitle>Split</letterboxd:filmTitle> <letterboxd:filmYear>2016</letterboxd:filmYear> <letterboxd:memberLike>No</letterboxd:memberLike> <tmdb:movieId>381288</tmdb:movieId> <description><![CDATA[ <p><img src="https://a.ltrbxd.com/resized/sm/upload/5e/pw/vs/km/8CHJcVc5IGXS1sC5DyjeMwTDEQH-0-600-0-900-crop.jpg?v=ab64d75800"/></p> <p>Watched on Sunday November 1, 2026.</p> ]]></description> <dc:creator>Nonresistant Really</dc:creator>"#;

    const REVIEW_ITEM: &str = r#" <title>Primetime, 2026 - ★★★★½</title> <link>https://letterboxd.com/othermember/film/primetime-2026/</link> <guid isPermaLink="false">letterboxd-review-1512336805</guid> <pubDate>Tue, 3 Nov 2026 07:37:59 +1300</pubDate> <letterboxd:watchedDate>2026-11-01</letterboxd:watchedDate> <letterboxd:rewatch>No</letterboxd:rewatch> <letterboxd:filmTitle>Primetime</letterboxd:filmTitle> <letterboxd:filmYear>2026</letterboxd:filmYear> <letterboxd:memberRating>4.5</letterboxd:memberRating> <letterboxd:memberLike>Yes</letterboxd:memberLike> <tmdb:movieId>1375441</tmdb:movieId> <description><![CDATA[ <p><img src="https://a.ltrbxd.com/resized/sm/upload/1/2/3/poster.jpg?v=abc"/></p> <p>Finally saw this and Wow!</p><p>Merritt Weber is so so good in this.</p> ]]></description> <dc:creator>Other Member</dc:creator>"#;

    const LIST_ITEM: &str = r#" <title>would sell my soul to star in</title> <link>https://letterboxd.com/maesnotebook/list/would-sell-my-soul-to-star-in/</link> <guid isPermaLink="false">letterboxd-list-42568585</guid> <pubDate>Tue, 6 Feb 2024 12:09:47 +1300</pubDate> <description><![CDATA[ <p>female parts I simply could never refuse</p> <ul> <li> <a href="https://letterboxd.com/film/gone-girl/">Gone Girl</a> </li> <li> <a href="https://letterboxd.com/film/la-la-land/">La La Land</a> </li> </ul> <p>...plus 86 more. <a href="https://letterboxd.com/maesnotebook/list/would-sell-my-soul-to-star-in/">View the full list on Letterboxd</a>.</p> ]]></description> <dc:creator>maesnotebook</dc:creator>"#;

    const LIKE_ITEM: &str = r#" <title>Film Name, 2024 - ★★★★</title> <link>https://letterboxd.com/othermember/film/film-name/</link> <guid isPermaLink="false">letterboxd-like-123456</guid> <pubDate>Mon, 2 Nov 2026 07:37:59 +1300</pubDate> <letterboxd:filmTitle>Film Name</letterboxd:filmTitle> <letterboxd:filmYear>2024</letterboxd:filmYear> <letterboxd:memberRating>4.0</letterboxd:memberRating> <description><![CDATA[ <p><img src="https://a.ltrbxd.com/resized/sm/upload/poster.jpg?v=abc"/></p> <p>Really lovely film.</p> ]]></description> <dc:creator>Other Member</dc:creator>"#;

    #[test]
    fn parses_a_watch_item() {
        let item = parse_item(WATCH_ITEM)
            .expect("valid xml")
            .expect("known guid");
        let FeedItem::Diary(entry) = item else {
            panic!("expected a diary entry");
        };

        assert_eq!(entry.guid, "letterboxd-watch-1567593051");
        assert_eq!(entry.title, "Split, 2016");
        assert_eq!(
            entry.link,
            "https://letterboxd.com/notreallyfilms/film/split-2016/"
        );
        assert_eq!(
            entry.pub_date.as_deref(),
            Some("Mon, 2 Nov 2026 07:37:59 +1300")
        );
        assert_eq!(entry.author.as_deref(), Some("Nonresistant Really"));
        assert_eq!(entry.film_title.as_deref(), Some("Split"));
        assert_eq!(entry.film_year, Some(2016));
        assert_eq!(entry.liked, Some(false));
        assert_eq!(entry.rewatch, Some(false));
        assert_eq!(entry.rating, None);
        assert_eq!(entry.tmdb_id, Some(TmdbId::Movie(381288)));
        assert_eq!(
            entry.watched_date.map(|date| date.to_string()),
            Some("2026-11-01".to_string())
        );
        assert_eq!(
            entry.poster_url.as_deref(),
            Some(
                "https://a.ltrbxd.com/resized/sm/upload/5e/pw/vs/km/8CHJcVc5IGXS1sC5DyjeMwTDEQH-0-600-0-900-crop.jpg?v=ab64d75800"
            )
        );
        assert_eq!(entry.review_text, None);
        assert_eq!(entry.review_html, None);
    }

    #[test]
    fn parses_a_review_item_with_a_rating() {
        let item = parse_item(REVIEW_ITEM)
            .expect("valid xml")
            .expect("known guid");
        let FeedItem::Diary(entry) = item else {
            panic!("expected a diary entry");
        };

        assert_eq!(entry.guid, "letterboxd-review-1512336805");
        assert_eq!(entry.rating.map(|rating| rating.value()), Some(4.5));
        assert_eq!(entry.liked, Some(true));
        assert_eq!(entry.film_title.as_deref(), Some("Primetime"));
        assert_eq!(entry.film_year, Some(2026));
        assert_eq!(entry.tmdb_id, Some(TmdbId::Movie(1375441)));

        let review = entry.review_text.expect("a review");
        assert!(review.contains("Finally saw this and Wow!"));
        assert!(review.contains("Merritt Weber is so so good in this."));
        assert!(!review.contains("img"));

        let html = entry.review_html.expect("the raw review html");
        assert!(html.contains("<p>Finally saw this and Wow!</p>"));
        assert!(html.contains("<img"));
    }

    #[test]
    fn parses_a_list_item() {
        let item = parse_item(LIST_ITEM)
            .expect("valid xml")
            .expect("known guid");
        let FeedItem::List(list) = item else {
            panic!("expected a list");
        };

        assert_eq!(list.guid, "letterboxd-list-42568585");
        assert_eq!(list.title, "would sell my soul to star in");
        assert_eq!(list.author.as_deref(), Some("maesnotebook"));
        assert_eq!(list.truncated_count, Some(86));
        assert_eq!(list.films.len(), 2);

        assert_eq!(list.films[0].title, "Gone Girl");
        assert_eq!(
            list.films[0].url.as_deref(),
            Some("https://letterboxd.com/film/gone-girl/")
        );
        assert_eq!(list.films[0].position, 1);

        assert_eq!(list.films[1].title, "La La Land");
        assert_eq!(
            list.films[1].url.as_deref(),
            Some("https://letterboxd.com/film/la-la-land/")
        );
        assert_eq!(list.films[1].position, 2);

        let description = list.description_text.expect("a description");
        assert!(description.contains("female parts I simply could never refuse"));
        assert!(!description.contains("plus 86 more"));
    }

    #[test]
    fn parses_a_like_item() {
        let item = parse_item(LIKE_ITEM)
            .expect("valid xml")
            .expect("known guid");
        let FeedItem::LikedReview(review) = item else {
            panic!("expected a liked review");
        };

        assert_eq!(review.guid, "letterboxd-like-123456");
        assert_eq!(review.title, "Film Name, 2024 - ★★★★");
        assert_eq!(review.author.as_deref(), Some("Other Member"));

        let film = review.film.expect("a film reference");
        assert_eq!(film.title, "Film Name");
        assert_eq!(film.year, Some(2024));

        let text = review.review_text.expect("a review");
        assert!(text.contains("Really lovely film."));

        let html = review.review_html.expect("the raw review html");
        assert!(html.contains("<p>Really lovely film.</p>"));
    }

    #[test]
    fn returns_none_for_an_unknown_guid() {
        let raw = r#" <title>Something</title> <link>https://example.com/</link> <guid isPermaLink="false">something-else-1</guid>"#;
        let item = parse_item(raw).expect("valid xml");
        assert!(item.is_none());
    }

    #[test]
    fn returns_none_without_a_guid() {
        let raw = r#" <title>Something</title> <link>https://example.com/</link>"#;
        let item = parse_item(raw).expect("valid xml");
        assert!(item.is_none());
    }

    #[test]
    fn decodes_entities_in_text_fields() {
        let raw = r#" <title>Who&#039;s Afraid of Virginia Woolf?</title> <link>https://example.com/</link> <guid isPermaLink="false">letterboxd-watch-1</guid> <dc:creator>annie&#039;s diary</dc:creator>"#;
        let item = parse_item(raw).expect("valid xml").expect("known guid");
        let FeedItem::Diary(entry) = item else {
            panic!("expected a diary entry");
        };

        assert_eq!(entry.title, "Who's Afraid of Virginia Woolf?");
        assert_eq!(entry.author.as_deref(), Some("annie's diary"));
    }

    #[test]
    fn parses_a_tv_identifier() {
        let raw = r#" <title>Over the Garden Wall, 2014 - ★★★★</title> <link>https://example.com/</link> <guid isPermaLink="false">letterboxd-watch-1068767517</guid> <letterboxd:rewatch>Yes</letterboxd:rewatch> <tmdb:tvId>61617</tmdb:tvId>"#;
        let item = parse_item(raw).expect("valid xml").expect("known guid");
        let FeedItem::Diary(entry) = item else {
            panic!("expected a diary entry");
        };

        assert_eq!(entry.tmdb_id, Some(TmdbId::Tv(61617)));
        assert_eq!(entry.rewatch, Some(true));
    }
}
