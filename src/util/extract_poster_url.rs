//! Extracts the poster image URL from a feed description.

/// Returns the poster URL from a Letterboxd description fragment.
///
/// Letterboxd embeds the poster as the first `<img src="...">` in the
/// description, pointing at a resized CDN copy of the TMDB poster.
///
/// ```
/// # use letterboxd_rss_client_rs::util::extract_poster_url;
/// let html = "<p><img src=\"https://a.ltrbxd.com/resized/x.jpg?v=1\"/></p>";
/// assert_eq!(extract_poster_url(html).as_deref(), Some("https://a.ltrbxd.com/resized/x.jpg?v=1"));
/// ```
pub fn extract_poster_url(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lower[cursor..].find("<img") {
        let start = cursor + offset;
        let tag_end = match lower[start..].find('>') {
            Some(end) => start + end,
            None => break,
        };
        let tag = &html[start..tag_end];

        if let Some(src) = attribute_value(tag, "src") {
            if !src.trim().is_empty() {
                return Some(src.trim().to_string());
            }
        }

        cursor = tag_end + 1;
    }

    None
}

/// Reads an attribute value out of an HTML tag.
fn attribute_value(tag: &str, attribute: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(offset) = lower[cursor..].find(attribute) {
        let position = cursor + offset;
        // The attribute must be preceded by whitespace to avoid matching
        // substrings such as `data-src` when looking for `src`.
        let preceded_by_space =
            position > 0 && lower.as_bytes()[position - 1].is_ascii_whitespace();
        let followed_by_equals = lower[position + attribute.len()..].starts_with('=');

        if preceded_by_space && followed_by_equals {
            let rest = &tag[position + attribute.len() + 1..];
            let rest_trimmed = rest.trim_start();
            let quote = rest_trimmed.chars().next()?;

            if quote == '"' || quote == '\'' {
                let body = &rest_trimmed[1..];
                let end = body.find(quote)?;
                return Some(body[..end].to_string());
            }

            let end = rest_trimmed
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(rest_trimmed.len());
            return Some(rest_trimmed[..end].to_string());
        }

        cursor = position + attribute.len();
    }

    None
}
