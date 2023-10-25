use std::fmt;
use std::str::FromStr;

use hmac::Mac;

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Duration, TimeZone, Utc};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use regex::Regex;

lazy_static::lazy_static! {
    // Base64 deserialization engine
    static ref BASE64_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
    // Regex for checking token strings
    static ref TOKEN_REGEX: Regex = Regex::new(r"^([\w_-]+).([\w_-]+)$").unwrap();
}

/// Various errors that can occur when handling tokens
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    // Token specific errors
    #[error("Token signature does not match")]
    SignatureMismatch,
    #[error("Token is expired")]
    Expired,
    #[error("Token is of invalid format")]
    InvalidFormat,
    // External errors
    #[error("Invalid HMAC message length")]
    InvalidLength(#[from] hmac::digest::InvalidLength),
    #[error("Invalid Utf8 string")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Serialization error")]
    Serde(#[from] serde_json::Error),
    #[error("Decode error")]
    DecodeError(#[from] base64::DecodeError),
}

/// Wrapper for token results
pub type TokenResult<T> = Result<T, TokenError>;

// A serialized token
#[derive(Debug, Clone)]
pub struct Token(String);

impl Token {
    /// Initialize a token builder to construct a token
    pub fn builder<T: Serialize>(payload: T) -> TokenBuilder<T> {
        TokenBuilder::new(payload)
    }

    /// Verify the token and deconstruct into the encoded payload value
    pub fn verify<T, K>(self, key: &K) -> TokenResult<T>
    where
        T: for<'de> Deserialize<'de>,
        K: Mac + Clone,
    {
        // Split the token string into it's base64 encoded components
        let (msg, sig) = self.split().ok_or(TokenError::InvalidFormat)?;
        // Decode the components
        let msg = BASE64_ENGINE.decode(msg)?;
        let sig = BASE64_ENGINE.decode(sig)?;
        // Verify and deserialize the message
        TokenMessage::verify_from_bytes(key, &msg, &sig)
    }

    fn split(self) -> Option<(String, String)> {
        let captures = TOKEN_REGEX.captures(&self.0)?;

        let msg = captures.get(1)?.as_str().to_string();
        let sig = captures.get(2)?.as_str().to_string();
        Some((msg, sig))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Token {
    type Err = TokenError;

    fn from_str(token: &str) -> TokenResult<Self> {
        if !TOKEN_REGEX.is_match(token) {
            Err(TokenError::InvalidFormat)
        } else {
            Ok(Self(token.to_string()))
        }
    }
}

/// Handy builder for creating and signing Tokens
#[derive(Debug)]
pub struct TokenBuilder<T> {
    expiration: Option<DateTime<Utc>>,
    payload: T,
}

impl<T: Serialize> TokenBuilder<T> {
    /// Create a new token builder with the specified payload
    pub fn new(payload: T) -> Self {
        Self {
            expiration: None,
            payload,
        }
    }
    /// Set the token to expire after a specified duration
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expiration = Some(Utc::now() + duration);
        self
    }
    /// Set the token to expire at a specified date-time
    pub fn expires_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.expiration = Some(timestamp);
        self
    }
    /// Sign the token with the specified key
    pub fn sign<K>(self, key: &K) -> TokenResult<Token>
    where
        K: Mac + Clone,
    {
        // Serialize the message to a string
        let msg = self.serialize_message()?;
        // Sign the message
        let sig = sign_message(key, &msg);
        // Base64 encode the two portions of the token
        let msg = BASE64_ENGINE.encode(msg);
        let sig = BASE64_ENGINE.encode(sig);
        // Combine to the final token string
        let token = format!("{}.{}", msg, sig);
        // Return the wrapped token
        Ok(Token(token))
    }

    fn serialize_message(self) -> serde_json::Result<String> {
        let msg: TokenMessage<T> = self.into();
        serde_json::to_string(&msg)
    }
}

/// Serializable structure for token messages
/// Contains the expiration timestamp and serializable payload
#[derive(Debug, Serialize, Deserialize)]
struct TokenMessage<T> {
    exp: Option<i64>,
    data: T,
}

impl<T: for<'de> Deserialize<'de>> TokenMessage<T> {
    /// Deserialize constructor for Token messages
    pub fn verify_from_bytes<K>(key: &K, msg: &[u8], signature: &[u8]) -> TokenResult<T>
    where
        K: Mac + Clone,
    {
        // Verify the message before deserialization
        verify_message(key, msg, signature)?;
        // Convert the bytes into a UTF8 string
        let msg = std::str::from_utf8(msg)?;
        // Deserialize from JSON
        let msg: TokenMessage<T> = serde_json::from_str(msg)?;
        // Check that the message is not expired
        if msg.is_expired() {
            Err(TokenError::Expired)
        } else {
            Ok(msg.data)
        }
    }

    /// Check if this token message is expired
    fn is_expired(&self) -> bool {
        self.exp
            // Map the utc timestamp into a UTC DateTime instance
            // NOTE: Default to the earliest date in ambiguous instances for security reasons
            .and_then(|exp| Utc.timestamp_opt(exp, 0u32).earliest())
            // Check if the current Utc timestamp is greater than the expiration
            .map(|exp| Utc::now() > exp)
            // Default to considering the token as expired if the timestamp is invalid
            .unwrap_or(false)
    }
}

/// Convert a TokenBuilder into a TokenMessage
impl<T> From<TokenBuilder<T>> for TokenMessage<T> {
    fn from(value: TokenBuilder<T>) -> Self {
        Self {
            exp: value.expiration.map(|d| d.timestamp()),
            data: value.payload,
        }
    }
}

/// Sign a message with a Key
fn sign_message<K>(key: &K, msg: &str) -> Vec<u8>
where
    K: Mac + Clone,
{
    key.clone()
        .chain_update(msg.as_bytes())
        .finalize()
        .into_bytes()
        .to_vec()
}

/// Verify a signed message with a key
fn verify_message<K>(key: &K, msg: &[u8], signature: &[u8]) -> TokenResult<()>
where
    K: Mac + Clone,
{
    let message_signature = key.clone().chain_update(msg).finalize().into_bytes();
    // Verify that the hmac signature matches the passed signature
    if message_signature[..] != signature[..] {
        Err(TokenError::SignatureMismatch)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use hmac::Hmac;
    use sha2::Sha256;

    use super::*;

    type Key = Hmac<Sha256>;

    #[test]
    fn can_sign_token() {
        let id = 8080usize;
        let key = Key::new_from_slice(b"test_key").unwrap();

        let token = Token::builder(id)
            .expires_in(Duration::minutes(5))
            .sign(&key)
            .expect("Failed to sign token");

        let token_value: usize = assert_ok!(token.verify(&key));
        assert_eq!(token_value, id);
    }

    #[test]
    fn non_expiry_tokens() {
        let id = 8080usize;
        let key = Key::new_from_slice(b"test_key").unwrap();

        let token = Token::builder(id).sign(&key).expect("Failed to sign token");
        assert_ok!(token.verify::<usize, Key>(&key));
    }

    #[test]
    fn fail_on_expired_in() {
        let id = 8080usize;
        let key = Key::new_from_slice(b"test_key").unwrap();

        let token = Token::builder(id)
            .expires_in(Duration::minutes(0))
            .sign(&key)
            .expect("Failed to sign token");

        assert_err!(token.verify::<usize, Key>(&key));
    }

    #[test]
    fn fail_on_expired_at() {
        let id = 8080usize;
        let key = Key::new_from_slice(b"test_key").unwrap();

        let token = Token::builder(id)
            .expires_at(Utc::now())
            .sign(&key)
            .expect("Failed to sign token");

        assert_err!(token.verify::<usize, Key>(&key));
    }

    #[test]
    fn fail_on_wrong_type() {
        let id = 8080usize;
        let key = Key::new_from_slice(b"test_key").unwrap();

        let token = Token::builder(id)
            .expires_in(Duration::minutes(5))
            .sign(&key)
            .expect("Failed to sign token");

        assert_err!(token.verify::<String, Key>(&key));
    }
}
