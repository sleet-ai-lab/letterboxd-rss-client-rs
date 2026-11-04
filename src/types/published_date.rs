//! Parses RFC 2822 `pubDate` timestamps into a richer representation.

use serde::{Deserialize, Serialize};

/// A parsed RSS `pubDate` timestamp.
///
/// Letterboxd emits timestamps such as `Mon, 2 Nov 2026 07:37:59 +1300`. The
/// offset is preserved, because the value is the member's local time rather
/// than a normalised UTC instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedDate {
    offset_seconds: i32,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl PublishedDate {
    /// Parses an RFC 2822 date string.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::InvalidDate`](crate::error::LetterboxdError::InvalidDate)
    /// when the value cannot be parsed.
    pub fn parse(value: &str) -> crate::Result<Self> {
        let trimmed = value.trim();
        let parsed =
            time::OffsetDateTime::parse(trimmed, &time::format_description::well_known::Rfc2822)
                .map_err(|_| crate::LetterboxdError::InvalidDate {
                    value: value.to_string(),
                    expected: "an RFC 2822 timestamp",
                })?;

        Ok(Self {
            offset_seconds: parsed.offset().whole_seconds(),
            year: parsed.year(),
            month: parsed.month() as u8,
            day: parsed.day(),
            hour: parsed.hour(),
            minute: parsed.minute(),
            second: parsed.second(),
        })
    }

    /// Returns the year component.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month component, from `1` to `12`.
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day component, from `1` to `31`.
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Returns the hour component, from `0` to `23`.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the minute component, from `0` to `59`.
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns the second component, from `0` to `59`.
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Returns the UTC offset of the timestamp in seconds.
    pub fn offset_seconds(&self) -> i32 {
        self.offset_seconds
    }

    /// Returns the UTC offset of the timestamp.
    pub fn offset(&self) -> time::UtcOffset {
        time::UtcOffset::from_whole_seconds(self.offset_seconds).unwrap_or(time::UtcOffset::UTC)
    }

    /// Returns the timestamp rendered as an RFC 2822 string.
    pub fn to_rfc2822(&self) -> String {
        let month = time::Month::try_from(self.month).unwrap_or(time::Month::January);
        let date = time::Date::from_calendar_date(self.year, month, self.day).unwrap_or(
            time::Date::from_calendar_date(1970, time::Month::January, 1).expect("valid date"),
        );
        let time_of_day = time::Time::from_hms(self.hour, self.minute, self.second)
            .unwrap_or(time::Time::MIDNIGHT);
        let offset_date_time =
            time::OffsetDateTime::new_in_offset(date, time_of_day, self.offset());
        offset_date_time
            .format(&time::format_description::well_known::Rfc2822)
            .unwrap_or_default()
    }
}
