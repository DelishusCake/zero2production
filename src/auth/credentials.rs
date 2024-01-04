use actix_web::http::header::{self, HeaderMap};

use anyhow::Context;

use secrecy::Secret;

const BASIC_AUTH_PREFIX: &str = "Basic ";

#[derive(Debug, Clone)]
pub enum Credentials {
    Basic {
        username: String,
        password: Secret<String>,
    },
}

impl Credentials {
    /// Extract credentials from the headers of a request
    pub fn from_headers(headers: &HeaderMap) -> anyhow::Result<Self> {
        // Get the authorization header value from the map
        let header_value = headers
            .get(header::AUTHORIZATION)
            .context("Missing authorization in header")?
            .to_str()?;
        // Match based on the prefix
        // TODO: Bearer auth?
        if header_value.starts_with(BASIC_AUTH_PREFIX) {
            Self::from_basic(header_value)
        } else {
            anyhow::bail!("Missing or unknown Authorization scheme")
        }
    }

    /// Extract credentials from a string formatted as 'Basic <base64 credentials>'
    pub fn from_basic(header_value: &str) -> anyhow::Result<Self> {
        // Strip the 'basic' prefix from the header
        let header_value = header_value
            .strip_prefix(BASIC_AUTH_PREFIX)
            .context("Authorization scheme not basic")?;
        // Base64 decode the credential string
        let decoded_value =
            string_from_base64(header_value).context("Failed to decode authorization header")?;
        // Split the string by the colon, extract the credentials
        let mut matches = decoded_value.splitn(2, ':');
        let username = matches.next().context("Missing email in authorization")?;
        let password = matches
            .next()
            .context("Missing password in authorization")?;

        Ok(Self::Basic {
            username: username.into(),
            password: Secret::new(password.into()),
        })
    }
}

fn string_from_base64(value: &str) -> anyhow::Result<String> {
    use base64::Engine;
    let value = base64::engine::general_purpose::STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::HeaderValue;
    use secrecy::ExposeSecret;

    #[test]
    fn can_parse_basic_authorization_from_headers() {
        let username = "test_username";
        let password = "test_password";

        let header_value = generate_basic_authorization(username, password);
        let header_value =
            HeaderValue::from_str(&header_value).expect("Failed to create header value");

        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header_value);

        let creds = Credentials::from_headers(&headers).expect("Failed to parse headers");

        check_basic(&creds, username, password);
    }

    #[test]
    fn can_parse_basic_authorization() {
        let username = "test_username";
        let password = "test_password";

        let basic_auth = generate_basic_authorization(username, password);

        let creds = Credentials::from_basic(&basic_auth).expect("Failed to parse headers");

        check_basic(&creds, username, password);
    }

    fn check_basic(creds: &Credentials, username: &str, password: &str) {
        match creds {
            Credentials::Basic {
                username: cred_username,
                password: cred_password,
            } => {
                assert_eq!(username, cred_username);
                assert_eq!(password, cred_password.expose_secret());
            }
        }
    }

    fn generate_basic_authorization(username: &str, password: &str) -> String {
        use base64::Engine;

        let username_password = format!("{}:{}", username, password);
        let username_password = base64::engine::general_purpose::STANDARD.encode(username_password);

        format!("Basic {}", username_password)
    }
}
