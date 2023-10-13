use std::future::Future;
use std::net::TcpListener;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Router, Server};

use sqlx::PgPool;

use tower_http::trace::TraceLayer;

use crate::client::EmailClient;
use crate::controller::subscriptions;
use crate::crypto::SigningKey;
use crate::util;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub signing_key: Arc<SigningKey>,
    pub email_client: Arc<EmailClient>,
}

#[tracing::instrument(name = "Health check")]
async fn health_check() -> &'static str {
    "I am alive"
}

pub fn run(
    pool: PgPool,
    signing_key: SigningKey,
    email_client: EmailClient,
    listener: TcpListener,
) -> anyhow::Result<impl Future<Output = Result<(), hyper::Error>>> {
    let state = AppState {
        pool,
        signing_key: Arc::new(signing_key),
        email_client: Arc::new(email_client),
    };

    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscriptions::create))
        .route(
            "/subscriptions/confirm/:token_str",
            get(subscriptions::confirm),
        )
        .with_state(state);

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(util::shutdown_signal());
    Ok(server)
}
