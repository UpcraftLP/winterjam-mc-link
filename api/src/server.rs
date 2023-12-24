use std::env;
use std::future::ready;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Mutex;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Scope, web};
use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::header::HeaderValue;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use listenfd::ListenFd;
use sea_orm::DatabaseConnection;
use rusty_interaction::handler::InteractionHandler;
use crate::{api, discord, health};
use crate::status::err_not_found;

static mut API_KEY: Option<HeaderValue> = None;

fn admin_api_key() -> &'static Option<HeaderValue> {
    unsafe {
        return &API_KEY;
    }
}

pub async fn server_main(db: DatabaseConnection) -> anyhow::Result<()> {
    unsafe {
        API_KEY = env::var("API_KEY").ok().map(|key| HeaderValue::from_str(key.as_str()).ok()).flatten();
    }

    let discord_handler = discord::init(db.clone()).await?;

    let mut listen_fd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(discord_handler.client().clone()))
            .default_service(web::route().to(default_route))
            .configure(|cfg| init(cfg, &discord_handler))
    });

    server = match listen_fd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server
            //TODO listen on ipv6 as well
            .bind((Ipv4Addr::UNSPECIFIED, 3000))?,
    };

    server.run().await?;

    Ok(())
}

fn init(cfg: &mut web::ServiceConfig, discord_handler: &InteractionHandler) {
    cfg.service(health::healthcheck);

    cfg.service(
        Scope::new("/api")
            .wrap_fn(|req, srv| {
                if let Some(expected_key) = admin_api_key() {
                    if let Some(provided_key) = req.headers().get("x-api-key") {
                        if provided_key == expected_key {
                            return srv.call(req);
                        }
                    }
                }

                Box::pin(ready(Ok(ServiceResponse::new(
                    req.into_parts().0,
                    HttpResponse::Unauthorized().finish(),
                ))))
            })
            .service(api::get_users)
            .service(api::get_user)
    );
    let discord_data = web::Data::new(Mutex::new(discord_handler.clone()));
    cfg.service(
        Scope::new("/discord")
            .app_data(discord_data)
            .route(
                "/interactions",
                web::post().to(
                    |data: web::Data<Mutex<InteractionHandler>>, req: HttpRequest, body: String| async move {
                        data.lock().unwrap().interaction(req, body).await
                    },
                ),
            )
    );
}

async fn default_route() -> HttpResponse {
    err_not_found()
}