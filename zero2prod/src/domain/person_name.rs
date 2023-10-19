use std::collections::HashSet;
use std::str::FromStr;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::{Result, Error};

const MAX_LEN: usize = 256;

#[derive(Debug, Clone)]
pub struct PersonName(String);

impl AsRef<str> for PersonName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for PersonName {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        lazy_static::lazy_static! {
            static ref INVALID_CHARS: HashSet<char> = vec!['/', '(', ')', '"', '<', '>', '\\', '{', '}']
                .into_iter()
                .collect();
        }

        if value.trim().is_empty() {
            return Err(Error::ParsingError("Name cannot be empty".into()));
        }
        if value.graphemes(true).count() > MAX_LEN {
            return Err(Error::ParsingError("Name too long".into()));
        }
        if value.chars().any(|c| INVALID_CHARS.contains(&c)) {
            return Err(Error::ParsingError("Name contains invalid characters".into()));
        }
        Ok(Self(value.to_string()))
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
