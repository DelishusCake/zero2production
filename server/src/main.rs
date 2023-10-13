use std::net::TcpListener;

use sqlx::PgPool;

use server::app;
use server::client::EmailClient;
use server::crypto::SigningKey;
use server::settings::Settings;
use server::telemetry::{create_subscriber, set_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = create_subscriber("debug".into(), std::io::stdout);
    set_subscriber(subscriber)?;

    let settings = Settings::load()?;

    let pool = PgPool::connect_lazy_with(settings.database.with_db());

    let signing_key = SigningKey::new(settings.app.secret_key())?;

    let email_client = EmailClient::new(
        settings.email.sender(),
        settings.email.api_timeout(),
        settings.email.api_base_url(),
        settings.email.api_auth_token(),
    )?;

    let listener = TcpListener::bind(settings.app.addr())?;
    tracing::info!("Running app on: {}", listener.local_addr()?);

    app::run(pool, signing_key, email_client, listener)?.await?;
    Ok(())
}
