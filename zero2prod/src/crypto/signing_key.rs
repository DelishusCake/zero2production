use hmac::{Hmac, Mac};

use sha2::Sha256;

use secrecy::Secret;

#[derive(Clone)]
pub struct SigningKey(Hmac<Sha256>);

impl SigningKey {
    pub fn new(key: &Secret<String>) -> anyhow::Result<Self> {
        use secrecy::ExposeSecret;

        let hmac = Hmac::new_from_slice(key.expose_secret().as_bytes())?;

        Ok(Self(hmac))
    }
}

impl AsRef<Hmac<Sha256>> for SigningKey {
    fn as_ref(&self) -> &Hmac<Sha256> {
        &self.0
    }
}
