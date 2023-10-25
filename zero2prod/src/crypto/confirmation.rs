use serde::{Deserialize, Serialize};

use uuid::Uuid;

use super::{SigningKey, Token, TokenResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Confirmation(Uuid);

impl From<Confirmation> for Uuid {
    fn from(value: Confirmation) -> Uuid {
        value.0
    }
}

impl From<Uuid> for Confirmation {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl Confirmation {
    pub fn sign(&self, key: &SigningKey) -> TokenResult<Token> {
        Token::builder(self.0).sign(key.as_ref())
    }

    pub fn verify(key: &SigningKey, token: &str) -> TokenResult<Self> {
        token.parse::<Token>()?.verify(key.as_ref())
    }
}
