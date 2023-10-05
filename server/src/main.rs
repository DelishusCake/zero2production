use std::net::TcpListener;

use anyhow::Context;

use sqlx::PgPool;

use server::app;
use server::settings::Settings;
use server::telemetry::{create_subscriber, set_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = create_subscriber("debug".into(), std::io::stdout);
    set_subscriber(subscriber)?;

    let settings = Settings::load()?;

    let pool = PgPool::connect_with(settings.database.with_db()).await?;

    let listener = TcpListener::bind(settings.app.addr())?;
    tracing::info!("Running app on: {}", listener.local_addr()?);

    app::run(pool, listener)?.await.context("Failed to run app")
}
