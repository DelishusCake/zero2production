use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{web, App, HttpServer};

use sqlx::PgPool;

use tracing_actix_web::TracingLogger;

use crate::client::EmailClient;
use crate::controller::{newsletters, subscriptions};
use crate::crypto::SigningKey;

/// Simple health-check endpoint
#[tracing::instrument(name = "Health check")]
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("I am alive")
}

/// Run the application on a specified TCP listener
pub fn run(
    listener: TcpListener,
    pool: PgPool,
    signing_key: SigningKey,
    email_client: EmailClient,
) -> anyhow::Result<Server> {
    // Wrap application data
    let pool = web::Data::new(pool);
    let signing_key = web::Data::new(signing_key);
    let email_client = web::Data::new(email_client);

    // Start the server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .app_data(signing_key.clone())
            .app_data(email_client.clone())
            .service(health_check)
            .service(subscriptions::scope())
            .service(newsletters::scope())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
