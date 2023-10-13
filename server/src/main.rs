use std::net::TcpListener;
use std::sync::Arc;

use sqlx::PgPool;

use server::app::{self, AppState};
use server::crypto::Crypto;
use server::settings::Settings;
use server::telemetry::{create_subscriber, set_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    let subscriber = create_subscriber("debug".into(), std::io::stdout);
    set_subscriber(subscriber)?;

    let settings = Settings::load()?;

    let pool = PgPool::connect_lazy_with(settings.database.with_db());

    let crypto = Crypto::new(settings.app.secret_key.expose_secret())?;
    let crypto = Arc::new(crypto);

    let state = AppState { pool, crypto };

    let listener = TcpListener::bind(settings.app.addr())?;
    tracing::info!("Running app on: {}", listener.local_addr()?);

    app::run(state, listener)?.await?;
    Ok(())
}
