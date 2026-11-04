//! The date a member watched a film.

use serde::{Deserialize, Serialize};

/// A calendar date on which a film was watched.
///
/// Letterboxd emits `watchedDate` as an ISO `YYYY-MM-DD` date with no time
/// component, which is why this type wraps [`time::Date`] rather than a
/// timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WatchedDate {
    date: time::Date,
}

impl WatchedDate {
    /// Parses an ISO `YYYY-MM-DD` date string.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::InvalidDate`](crate::error::LetterboxdError::InvalidDate)
    /// when the value is not a valid date.
    pub fn parse(value: &str) -> crate::Result<Self> {
        let date = time::Date::parse(
            value,
            &time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
                .expect("valid format"),
        )
        .map_err(|_| crate::LetterboxdError::InvalidDate {
            value: value.to_string(),
            expected: "an ISO 8601 calendar date (YYYY-MM-DD)",
        })?;
        Ok(Self { date })
    }

    /// Returns the underlying [`time::Date`].
    pub fn as_date(&self) -> time::Date {
        self.date
    }

    /// Returns the year component.
    pub fn year(&self) -> i32 {
        self.date.year()
    }

    /// Returns the month component, from `1` to `12`.
    pub fn month(&self) -> u8 {
        self.date.month() as u8
    }

    /// Returns the day component, from `1` to `31`.
    pub fn day(&self) -> u8 {
        self.date.day()
    }
}

impl std::fmt::Display for WatchedDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso_string())
    }
}

impl WatchedDate {
    /// Returns the date as an ISO `YYYY-MM-DD` string.
    pub fn to_iso_string(&self) -> String {
        let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
            .expect("valid format");
        self.date.format(&format).unwrap_or_default()
    }
}

impl TryFrom<String> for WatchedDate {
    type Error = crate::LetterboxdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        WatchedDate::parse(&value)
    }
}

impl From<WatchedDate> for String {
    fn from(date: WatchedDate) -> String {
        date.to_string()
    }
}
