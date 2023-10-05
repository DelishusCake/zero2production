use std::net::TcpListener;

use anyhow::Context;

use sqlx::PgPool;

use server::app;
use server::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load().expect("Failed to load settings");

    let pool = PgPool::connect_with(settings.database.with_db()).await?;

    let listener = TcpListener::bind(settings.app.addr())?;

    app::run(pool, listener)?.await.context("Failed to run app")
}
