use hmac::{Hmac, Mac};

use jwt::{SigningAlgorithm, VerifyingAlgorithm};

use sha2::Sha256;

use secrecy::Secret;

use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Clone)]
pub struct SigningKey(Hmac<Sha256>);

impl SigningKey {
    pub fn new(key: &Secret<String>) -> anyhow::Result<Self> {
        use secrecy::ExposeSecret;

        let hmac = Hmac::new_from_slice(key.expose_secret().as_bytes())?;

        Ok(Self(hmac))
    }
}

impl AsRef<dyn SigningAlgorithm> for SigningKey {
    fn as_ref(&self) -> &(dyn SigningAlgorithm + 'static) {
        &self.0
    }
}

impl AsRef<dyn VerifyingAlgorithm> for SigningKey {
    fn as_ref(&self) -> &(dyn VerifyingAlgorithm + 'static) {
        &self.0
    }
}

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
    pub fn sign(&self, key: &SigningKey) -> Result<String, jwt::Error> {
        use jwt::SignWithKey;

        self.0.sign_with_key(key)
    }

    pub fn verify(key: &SigningKey, token: &str) -> Result<Self, jwt::Error> {
        use jwt::VerifyWithKey;

        let claims = token.verify_with_key(key)?;

        Ok(Self(claims))
    }
}
