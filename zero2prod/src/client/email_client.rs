use std::time::Duration;

use reqwest::Client;

use serde::Serialize;

use secrecy::Secret;

use url::Url;

use crate::domain::EmailAddress;

const POSTMARK_TOKEN_HEADER: &str = "X-Postmark-Server-Token";

#[derive(Debug)]
pub struct EmailClient {
    client: Client,
    sender: EmailAddress,

    api_send_email_url: Url,
    api_auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        sender: EmailAddress,
        api_timeout: Duration,
        api_base_url: Url,
        api_auth_token: Secret<String>,
    ) -> anyhow::Result<Self> {
        let client = Client::builder().timeout(api_timeout).build()?;

        let api_send_email_url = api_base_url.join("email")?;

        Ok(Self {
            client,
            sender,
            api_send_email_url,
            api_auth_token,
        })
    }

    #[tracing::instrument(name = "Send an email via API")]
    pub async fn send(&self, email: Email) -> reqwest::Result<()> {
        use secrecy::ExposeSecret;

        let body = email.as_request(&self.sender);

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
pub struct Email {
    pub recipient: EmailAddress,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

impl Email {
    fn as_request<'e>(&'e self, sender: &'e EmailAddress) -> SendEmailRequest<'e> {
        SendEmailRequest {
            to: self.recipient.as_ref(),
            from: sender.as_ref(),
            subject: &self.subject,
            html_body: &self.html_body,
            text_body: &self.text_body,
        }
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

        assert_ok!(client.send(fake_email()).await);
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

        assert_err!(client.send(fake_email()).await);
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

        assert_err!(client.send(fake_email()).await);
    }

    fn fake_email_address() -> EmailAddress {
        SafeEmail().fake::<String>().parse().unwrap()
    }

    fn fake_email() -> Email {
        let recipient = fake_email_address();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        Email {
            recipient,
            subject,
            html_body: content.clone(),
            text_body: content.clone(),
        }
    }

    fn email_client(server_uri: &str) -> EmailClient {
        let sender = fake_email_address();
        let mock_api_timeout = Duration::from_secs(2);
        let mock_api_url = Url::parse(server_uri).unwrap();
        let mock_api_auth: Secret<String> = Faker.fake::<String>().parse().unwrap();

        EmailClient::new(sender, mock_api_timeout, mock_api_url, mock_api_auth).unwrap()
    }
}
