use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, HttpResponse, Responder};

use serde::Deserialize;

use sqlx::PgPool;

#[derive(Debug, Deserialize)]
struct NewSubscriberForm {
    name: String,
    email: String,
}

#[post("")]
async fn create(form: web::Form<NewSubscriberForm>, pool: web::Data<PgPool>) -> impl Responder {
    match insert_subscriber(&pool, &form.name, &form.email).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn insert_subscriber(pool: &PgPool, name: &str, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "insert into subscriptions(name, email) values ($1, $2)",
        name,
        email
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/subscriptions").service(create)
}
