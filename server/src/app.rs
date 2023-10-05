use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{web, App, HttpServer};

use sqlx::PgPool;

use tracing_actix_web::TracingLogger;

use crate::controller::subscriptions;

#[tracing::instrument(name = "Health check")]
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(pool: PgPool, listener: TcpListener) -> std::io::Result<Server> {
    let pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .service(health_check)
            .service(subscriptions::scope())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
