use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
    version: &'a str,
}

#[get("/_health")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        version: option_env!("VERSION").unwrap_or("unknown"),
    })
}