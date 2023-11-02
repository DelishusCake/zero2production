use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::client::EmailClient;
use zero2prod::crypto::SigningKey;

use server::app;
use server::settings::Settings;
use server::telemetry::{create_subscriber, set_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create and set the logging subscriber.
    // Use the std out for writing and "debug" as the default log level
    let subscriber = create_subscriber("debug".into(), std::io::stdout);
    set_subscriber(subscriber)?;
    // Load the application settings for the current environment
    let settings = Settings::load()?;
    // Set up a lazy database connection to Postgres
    let pool = PgPool::connect_lazy_with(settings.database.with_db());
    // Set up a cryptographic signing key
    // TODO: Consider secret key rotation and support for historical key?
    let signing_key = SigningKey::new(settings.app.secret_key())?;
    // Set up an email client
    let email_client = EmailClient::new(
        settings.email.sender(),
        settings.email.api_timeout(),
        settings.email.api_base_url(),
        settings.email.api_auth_token(),
    )?;
    // Start listening on the application address
    let listener = TcpListener::bind(settings.app.addr())?;
    tracing::info!("Running app on: {}", listener.local_addr()?);
    // Run the application server
    app::run(listener, pool, signing_key, email_client)?.await?;
    Ok(())
}
