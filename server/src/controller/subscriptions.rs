use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, HttpResponse, Responder};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NewSubscriberForm {
    name: String,
    email: String,
}

#[post("")]
async fn create(_form: web::Form<NewSubscriberForm>) -> impl Responder {
    HttpResponse::Ok()
}

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/subscriptions").service(create)
}
