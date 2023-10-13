use std::net::TcpListener;
use std::sync::Arc;

use sqlx::PgPool;

use server::app::{self, AppState};
use server::crypto::SigningKey;
use server::settings::Settings;
use server::telemetry::{create_subscriber, set_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    let subscriber = create_subscriber("debug".into(), std::io::stdout);
    set_subscriber(subscriber)?;

    let settings = Settings::load()?;

    let pool = PgPool::connect_lazy_with(settings.database.with_db());

    let signing_key = SigningKey::new(settings.app.secret_key.expose_secret())?;
    let signing_key = Arc::new(signing_key);

    let state = AppState { pool, signing_key };

    let listener = TcpListener::bind(settings.app.addr())?;
    tracing::info!("Running app on: {}", listener.local_addr()?);

    app::run(state, listener)?.await?;
    Ok(())
}
