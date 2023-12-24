use actix_web::HttpResponse;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize)]
struct StatusResult<'a> {
    pub status: &'a str,
    pub message: Option<&'a str>,
}

pub(crate) fn success() -> HttpResponse {
    HttpResponse::Ok().json(StatusResult {
        status: "ok",
        message: None,
    })
}

pub(crate) fn err_not_found() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

pub(crate) fn err_server(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(StatusResult {
        status: "error",
        message: Some(message),
    })
}