mod server;
mod health;
mod status;
mod discord;
mod admin;
mod mojang;
mod api;

use std::env;
use std::time::Duration;
use sea_orm::{ConnectOptions, Database};
use migration::{Migrator, MigratorTrait};

pub async fn start() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut opts = ConnectOptions::new(db_url);
    opts.max_connections(3)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(1200))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    let db = Database::connect(opts).await?;
    Migrator::up(&db, None).await?;

    server::server_main(db.clone()).await?;

    Ok(())
}
