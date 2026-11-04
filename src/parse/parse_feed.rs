//! Parses a Letterboxd RSS document into a [`LetterboxdFeed`].

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::LetterboxdError;
use crate::parse::parse_item::parse_item;
use crate::result::Result;
use crate::types::{FeedItem, LetterboxdFeed};

/// Parses a Letterboxd RSS document into a [`LetterboxdFeed`].
///
/// # Errors
///
/// Returns [`LetterboxdError::Xml`] when the document is not well-formed XML,
/// and [`LetterboxdError::MalformedFeed`] when it is not an RSS document with
/// a `<channel>`.
///
/// ```
/// # use letterboxd_rss_client_rs::parse::parse_feed;
/// let xml = r#"<rss version="2.0"><channel><title>Letterboxd - Test</title>
/// <link>https://letterboxd.com/test/</link></channel></rss>"#;
/// let feed = parse_feed(xml).expect("valid feed");
/// assert_eq!(feed.member_name(), "Test");
/// ```
pub fn parse_feed(xml: &str) -> Result<LetterboxdFeed> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut title = String::new();
    let mut link = String::new();
    let mut description = None;
    let mut atom_link = None;
    let mut items: Vec<FeedItem> = Vec::new();

    // The name of the channel-level element currently being read, if any.
    let mut current: Option<String> = None;
    let mut current_text = String::new();

    // Depth of the element nesting, used to recognise the direct children of
    // `<channel>`, which are the elements holding the channel's metadata.
    let mut depth = 0usize;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let raw_name = event.name();
                let name = local_name(raw_name.as_ref());

                if name == "item" {
                    // Capture the raw XML of the item so that it can be handed
                    // to `parse_item` unchanged.
                    let span = reader.read_to_end(raw_name).map_err(LetterboxdError::Xml)?;
                    let bytes = xml.as_bytes();
                    let start = (span.start as usize).min(bytes.len());
                    let end = (span.end as usize).min(bytes.len());
                    let raw = String::from_utf8_lossy(&bytes[start..end]);
                    if let Some(item) = parse_item(&raw)? {
                        items.push(item);
                    }
                    continue;
                }

                // Channel-level elements are the direct children of `<channel>`;
                // their own children are not channel fields.
                if depth == 2 && current.is_none() {
                    current = Some(name);
                    current_text.clear();
                }

                depth += 1;
            }
            Ok(Event::End(event)) => {
                let name = local_name(event.name().as_ref());
                depth = depth.saturating_sub(1);

                if depth == 2 {
                    if let Some(expected) = current.take() {
                        if name == expected {
                            let value = std::mem::take(&mut current_text);
                            apply_channel_field(
                                &expected,
                                &value,
                                &mut title,
                                &mut link,
                                &mut description,
                                &mut atom_link,
                            );
                        }
                        current_text.clear();
                    }
                }
            }
            Ok(Event::Empty(event)) => {
                let raw_name = event.name();
                let name = local_name(raw_name.as_ref());

                // `<atom:link .../>` carries its target in the `href`
                // attribute rather than in its text.
                if name == "link" && raw_name.as_ref().starts_with(b"atom:") && atom_link.is_none()
                {
                    atom_link = attribute_value(&event, b"href");
                }
            }
            Ok(Event::CData(data)) => {
                let text = String::from_utf8_lossy(data.as_ref()).into_owned();
                current_text.push_str(&text);
            }
            Ok(Event::Text(text)) => {
                let value = text.xml_content().unwrap_or_default().into_owned();
                current_text.push_str(&value);
            }
            Ok(Event::GeneralRef(reference)) => {
                if let Ok(Some(ch)) = reference.resolve_char_ref() {
                    current_text.push(ch);
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(LetterboxdError::Xml(error)),
            _ => {}
        }
    }

    if title.is_empty() && link.is_empty() && items.is_empty() {
        return Err(LetterboxdError::MalformedFeed(
            "document has no <channel> contents".to_string(),
        ));
    }

    Ok(LetterboxdFeed {
        title,
        link,
        description,
        atom_link,
        items,
    })
}

/// Reads the value of an attribute of a start tag.
fn attribute_value(event: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if attribute.key.as_ref() == name {
            return Some(String::from_utf8_lossy(&attribute.value).into_owned());
        }
    }
    None
}

fn apply_channel_field(
    name: &str,
    value: &str,
    title: &mut String,
    link: &mut String,
    description: &mut Option<String>,
    atom_link: &mut Option<String>,
) {
    match name {
        "title" => *title = value.to_string(),
        "link" => {
            if link.is_empty() {
                *link = value.to_string();
            }
        }
        "description" => {
            if description.is_none() {
                *description = Some(value.to_string());
            }
        }
        "atom:link" => *atom_link = Some(value.to_string()),
        _ => {}
    }
}

/// Strips a namespace prefix from an element name.
pub(crate) fn local_name(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    match text.split_once(':') {
        Some((_, local)) => local.to_string(),
        None => text.into_owned(),
    }
}
