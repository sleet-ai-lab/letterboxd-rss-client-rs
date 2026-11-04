//! Converts a fragment of HTML into plain text.

/// Converts an HTML fragment into readable plain text.
///
/// The function understands the small subset of HTML that Letterboxd emits in
/// RSS descriptions: `<p>`, `<br>`, `<ul>`/`<ol>`/`<li>`, `<a>` and `<img>`
/// elements, plus HTML entities. Block-level tags become line breaks so the
/// result keeps the shape of the original review.
///
/// ```
/// # use letterboxd_rss_client_rs::util::html_to_text;
/// assert_eq!(html_to_text("<p>Hello <b>world</b></p>"), "Hello world");
/// ```
pub fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut skip_until: Option<&str> = None;

    let bytes: Vec<char> = html.chars().collect();
    let mut index = 0usize;

    while index < bytes.len() {
        let ch = bytes[index];

        if let Some(tag) = skip_until {
            let rest: String = bytes[index..].iter().collect();
            if rest.to_ascii_lowercase().contains(tag) {
                out.push(' ');
                index += 1;
                continue;
            }
            if ch == '<' {
                // Look ahead for the closing tag.
                let rest_lower: String = bytes[index..]
                    .iter()
                    .collect::<String>()
                    .to_ascii_lowercase();
                let close = format!("</{tag}");
                if let Some(found) = rest_lower.find(&close) {
                    index += found;
                    skip_until = None;
                    continue;
                }
            }
            index += 1;
            continue;
        }

        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
                index += 1;
            }
            '>' if in_tag => {
                in_tag = false;
                let lower = tag_name.trim().to_ascii_lowercase();
                let name = lower.trim_start_matches('/').trim().to_string();

                if lower.starts_with("img") {
                    // Images contribute nothing to the text.
                    if !out.ends_with(' ') && !out.is_empty() {
                        out.push(' ');
                    }
                } else if lower.starts_with("br")
                    || name == "p"
                    || name == "li"
                    || name == "ul"
                    || name == "ol"
                    || name == "div"
                    || name == "blockquote"
                {
                    // A closing block tag ends a block rather than starting a
                    // new one, so it must not leave a trailing line break.
                    let is_closing = lower.starts_with('/');
                    if !is_closing {
                        push_newline(&mut out);
                    }
                }

                if name == "script" || name == "style" {
                    skip_until = Some(if name == "script" { "script" } else { "style" });
                }
                index += 1;
            }
            _ if in_tag => {
                tag_name.push(ch);
                index += 1;
            }
            _ => {
                out.push(ch);
                index += 1;
            }
        }
    }

    decode_entities(&out)
}

fn push_newline(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

fn decode_entities(text: &str) -> String {
    html_escape::decode_html_entities(text).into_owned()
}
