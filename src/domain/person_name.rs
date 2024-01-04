use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use unicode_segmentation::UnicodeSegmentation;

const MAX_LEN: usize = 256;

/// A user supplied name of a person.
#[derive(Debug, PartialEq, Clone)]
pub struct PersonName(String);

impl FromStr for PersonName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        lazy_static::lazy_static! {
            static ref INVALID_CHARS: HashSet<char> = vec!['/', '(', ')', '"', '<', '>', '\\', '{', '}']
                .into_iter()
                .collect();
        }

        if value.trim().is_empty() {
            return Err("Name cannot be empty".into());
        }
        if value.graphemes(true).count() > MAX_LEN {
            return Err("Name too long".into());
        }
        if value.chars().any(|c| INVALID_CHARS.contains(&c)) {
            return Err("Name contains invalid characters".into());
        }
        Ok(Self(value.to_string()))
    }
}

impl AsRef<str> for PersonName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use super::*;

    #[test]
    fn long_name_valid() {
        let name = "ё".repeat(MAX_LEN);
        assert_ok!(name.parse::<PersonName>());
    }

    #[test]
    fn too_long_name_valid() {
        let name = "ё".repeat(MAX_LEN + 10);
        assert_err!(name.parse::<PersonName>());
    }

    #[test]
    fn empty_name_valid() {
        let name = "";
        assert_err!(name.parse::<PersonName>());
    }

    #[test]
    fn blank_name_valid() {
        let name = "   ";
        assert_err!(name.parse::<PersonName>());
    }

    #[test]
    fn bad_chars_invalid() {
        let name = "test{}\\\"/<>";
        assert_err!(name.parse::<PersonName>());
    }
}
