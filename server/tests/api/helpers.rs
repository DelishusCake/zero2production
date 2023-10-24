use std::net::TcpListener;
use std::time::Duration;

use reqwest::{Client, Method, Response};

use sqlx::PgPool;

use secrecy::Secret;

use serde::Serialize;

use url::Url;

use wiremock::MockServer;

use zero2prod::client::EmailClient;
use zero2prod::crypto::SigningKey;

use server::app;

#[derive(Debug, Serialize)]
pub struct NewSubscriber {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub struct TestApp {
    addr: String,

    pub client: Client,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn spawn(pool: &PgPool) -> Self {
        use rand::{distributions::Alphanumeric, Rng};

        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen on random port");
        let port = listener.local_addr().unwrap().port();

        let addr = format!("http://127.0.0.1:{}", port);

        let signing_key = {
            let rand_key: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect();
            let rand_key = Secret::new(rand_key);

            SigningKey::new(&rand_key).expect("Failed to create crypto signing key")
        };

        let email_server = MockServer::start().await;

        let email_client = {
            let sender = "test@test.com"
                .parse()
                .expect("Failed to parse sender email address");
            let api_base_url =
                Url::parse(&email_server.uri()).expect("Failed to parse mock server uri");
            let api_auth_token = Secret::new("TestAuthorization".into());
            let api_timeout = Duration::from_secs(2);

            EmailClient::new(sender, api_timeout, api_base_url, api_auth_token)
                .expect("Failed to create email client")
        };

        let server = app::run(pool.clone(), signing_key, email_client, listener)
            .expect("Failed to spawn app instance");
        let _ = tokio::spawn(server);

        let client = Client::new();

        Self {
            addr,
            client,
            email_server,
        }
    }

    pub fn request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", &self.addr, url);
        self.client.request(method, url)
    }

    pub async fn health_check(&self) -> reqwest::Result<Response> {
        self.request(Method::GET, "health_check").send().await
    }

    pub async fn subscription_create(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> reqwest::Result<Response> {
        self.request(Method::POST, "subscriptions")
            .form(new_subscriber)
            .send()
            .await
    }
}
