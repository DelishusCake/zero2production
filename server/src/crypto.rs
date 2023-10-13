use hmac::{Hmac, Mac};

use sha2::Sha256;

use jwt::{SigningAlgorithm, VerifyingAlgorithm};

#[derive(Clone)]
pub struct SigningKey(Hmac<Sha256>);

impl SigningKey {
    pub fn new(secret: &str) -> anyhow::Result<Self> {
        let hmac = Hmac::new_from_slice(secret.as_bytes())?;
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
