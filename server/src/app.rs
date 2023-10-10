use std::future::Future;
use std::net::TcpListener;

use axum::routing::{get, post};
use axum::{Router, Server};

use sqlx::PgPool;

use tower_http::trace::TraceLayer;

use crate::util;

use crate::controller::subscriptions;

#[tracing::instrument(name = "Health check")]
async fn health_check() -> &'static str {
    "I am alive"
}

pub fn run(pool: PgPool, listener: TcpListener) -> anyhow::Result<impl Future<Output=Result<(), hyper::Error>>> {
    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscriptions::create))
        .with_state(pool);

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(util::shutdown_signal());
    Ok(server)
}
