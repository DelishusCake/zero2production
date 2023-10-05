use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{App, HttpServer};

use crate::controller::subscriptions;

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(subscriptions::scope())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
