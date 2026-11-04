//! The rating a member gave a film, expressed in half stars.

use serde::{Deserialize, Serialize};

/// A member rating on the Letterboxd half-star scale.
///
/// Letterboxd ratings run from `0.5` to `5.0` in half-star increments. The
/// [`Rating::stars`] method returns the filled-star glyphs, and
/// [`Rating::full_stars`] / [`Rating::half_star`] expose the components
/// separately.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Rating {
    value: f64,
}

impl Rating {
    /// The lowest valid rating.
    pub const MIN: f64 = 0.5;

    /// The highest valid rating.
    pub const MAX: f64 = 5.0;

    /// Creates a rating from a numeric value.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::InvalidRating`](crate::error::LetterboxdError::InvalidRating)
    /// when the value is outside `0.5..=5.0` or is not a multiple of `0.5`.
    pub fn new(value: f64) -> crate::Result<Self> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(crate::LetterboxdError::InvalidRating(value));
        }
        let doubled = (value * 2.0).round();
        if (doubled - value * 2.0).abs() > f64::EPSILON * 4.0 {
            return Err(crate::LetterboxdError::InvalidRating(value));
        }
        Ok(Self {
            value: doubled / 2.0,
        })
    }

    /// Returns the numeric rating value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the number of whole stars, from `0` to `5`.
    pub fn full_stars(&self) -> u8 {
        self.value.floor() as u8
    }

    /// Returns whether the rating carries a half star.
    pub fn half_star(&self) -> bool {
        (self.value * 2.0).round() as u8 % 2 == 1
    }

    /// Returns the rating rendered as star glyphs, for example `★★★★½`.
    pub fn stars(&self) -> String {
        let full = self.full_stars();
        let mut out = "★".repeat(full as usize);
        if self.half_star() {
            out.push('½');
        }
        out
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<f64> for Rating {
    type Error = crate::LetterboxdError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Rating::new(value)
    }
}

impl From<Rating> for f64 {
    fn from(rating: Rating) -> f64 {
        rating.value
    }
}
