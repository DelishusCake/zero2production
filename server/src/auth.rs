use std::future::Future;
use std::pin::Pin;

use actix_web::http::header::{self, HeaderMap};
use actix_web::{dev, FromRequest, HttpRequest};

use anyhow::Context;

use secrecy::Secret;

use crate::error::RestError;

#[derive(Debug)]
pub struct Administrator;

impl FromRequest for Administrator {
    type Error = RestError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let _credentials =
                Credentials::from_basic(req.headers()).map_err(RestError::FailedToAuthenticate)?;

            // TODO: Actually authenticate user
            Ok(Administrator)
        })
    }
}

#[derive(Debug)]
struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl Credentials {
    fn from_basic(headers: &HeaderMap) -> anyhow::Result<Self> {
        use base64::Engine;

        let header_value = headers
            .get(header::AUTHORIZATION)
            .context("Missing authorization in header")?
            .to_str()?
            .strip_prefix("Basic ")
            .context("Authorization scheme not basic")?;

        let decoded_value = base64::engine::general_purpose::STANDARD
            .decode(header_value)
            .context("Failed to decode authorization header")?;
        let decoded_value =
            String::from_utf8(decoded_value).context("Failed to decode authorization header")?;

        let mut matches = decoded_value.splitn(2, ':');
        let username = matches
            .next()
            .context("Missing username in authorization")?;
        let password = matches
            .next()
            .context("Missing password in authorization")?;

        Ok(Self {
            username: username.into(),
            password: Secret::new(password.into()),
        })
    }
}
