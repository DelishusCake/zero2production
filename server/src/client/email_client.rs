use std::convert::Infallible;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Context;

use reqwest::Client;

use serde::Serialize;

use secrecy::Secret;

use url::Url;

use zero2prod::domain::EmailAddress;

const POSTMARK_TOKEN_HEADER: &str = "X-Postmark-Server-Token";

#[derive(Debug)]
pub struct EmailClient {
    client: Client,
    sender: EmailAddress,

    api_send_email_url: Url,
    api_auth_token: EmailAuthorizationToken,
}

impl EmailClient {
    pub fn new(
        sender: EmailAddress,
        api_timeout: Duration,
        api_base_url: Url,
        api_auth_token: EmailAuthorizationToken,
    ) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(api_timeout)
            .build()
            .context("Failed to build http client")?;

        let api_send_email_url = api_base_url
            .join("email")
            .context("Failed to create send email endpoint URL")?;

        Ok(Self {
            client,
            sender,
            api_send_email_url,
            api_auth_token,
        })
    }

    pub async fn send(
        &self,
        recipient: EmailAddress,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> anyhow::Result<()> {
        use secrecy::ExposeSecret;

        let body = SendEmailRequest {
            to: recipient.as_ref(),
            from: self.sender.as_ref(),
            subject,
            html_body,
            text_body,
        };

        self.client
            .post(self.api_send_email_url.clone())
            .header(POSTMARK_TOKEN_HEADER, self.api_auth_token.expose_secret())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EmailAuthorizationToken(Secret<String>);

impl FromStr for EmailAuthorizationToken {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Infallible> {
        let value = value.to_string();
        let value = Secret::new(value);

        Ok(Self(value))
    }
}

impl From<Secret<String>> for EmailAuthorizationToken {
    fn from(value: Secret<String>) -> Self {
        Self(value)
    }
}

impl secrecy::ExposeSecret<String> for EmailAuthorizationToken {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    to: &'a str,
    from: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};

    use wiremock::matchers::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, req: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&req.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_posts_to_api() {
        let mock_server = MockServer::start().await;
        let client = email_client(&mock_server.uri());

        Mock::given(header_exists(POSTMARK_TOKEN_HEADER))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient = fake_email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        let res = client.send(recipient, &subject, &content, &content).await;

        assert_ok!(res);
    }

    #[tokio::test]
    async fn send_fails_if_api_returns_500() {
        let mock_server = MockServer::start().await;
        let client = email_client(&mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient = fake_email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        let res = client.send(recipient, &subject, &content, &content).await;

        assert_err!(res);
    }

    #[tokio::test]
    async fn send_fails_if_api_takes_too_long() {
        let mock_server = MockServer::start().await;
        let client = email_client(&mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient = fake_email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        let res = client.send(recipient, &subject, &content, &content).await;

        assert_err!(res);
    }

    fn fake_email() -> EmailAddress {
        SafeEmail().fake::<String>().parse().unwrap()
    }

    fn email_client(server_uri: &str) -> EmailClient {
        let sender = fake_email();
        let mock_api_timeout = Duration::from_secs(2);
        let mock_api_url = Url::parse(server_uri).unwrap();
        let mock_api_auth: EmailAuthorizationToken = Faker.fake::<String>().parse().unwrap();

        EmailClient::new(sender, mock_api_timeout, mock_api_url, mock_api_auth).unwrap()
    }
}
