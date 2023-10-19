use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{web, App, HttpServer};

use sqlx::PgPool;

use tracing_actix_web::TracingLogger;

use zero2prod::client::EmailClient;
use zero2prod::crypto::SigningKey;

use crate::controller::subscriptions;

#[tracing::instrument(name = "Health check")]
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("I am alive")
}

pub fn run(
    pool: PgPool,
    signing_key: SigningKey,
    email_client: EmailClient,
    listener: TcpListener,
) -> anyhow::Result<Server> {
    let pool = web::Data::new(pool);
    let signing_key = web::Data::new(signing_key);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .app_data(signing_key.clone())
            .app_data(email_client.clone())
            .service(health_check)
            .service(subscriptions::scope())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
