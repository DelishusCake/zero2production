use std::convert::Infallible;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OAuth2Provider {
    Google,
    Unknown(String),
}

impl From<String> for OAuth2Provider {
    fn from(value: String) -> Self {
        value.into()
    }
}

impl FromStr for OAuth2Provider {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();
        let value = match value.as_str() {
            "google" => Self::Google,
            value => {
                tracing::warn!("Attempt to parse unknown OAuth2 provider \"{}\"", value);
                Self::Unknown(value.to_string())
            },
        };
        Ok(value)
    }
}

impl AsRef<str> for OAuth2Provider {
    fn as_ref(&self) -> &str {
        match self {
            Self::Google => "google",
            Self::Unknown(value) => value.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_str_to_enum() {
        let values = vec![OAuth2Provider::Google];
        for value in values {
            let value_as_str: &str = value.as_ref();
            assert_eq!(value, value_as_str.parse().unwrap());
        }
    }
}
