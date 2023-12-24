use sea_orm::DatabaseConnection;
use rusty_interaction::{defer, slash_command};
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::interaction::{Context, InteractionResponse};
use rusty_interaction::types::Snowflake;
use webhook::Webhook;

use crate::discord::register::update_global_commands;

mod register;
mod commands;
mod webhook;

static mut OWNER_ID: Snowflake = 0;
pub(crate) static mut MODERATOR_ROLES: Vec<Snowflake> = Vec::new();
pub(crate) static mut GUILD_ID: Snowflake = 0;

pub(crate) async fn init(db: DatabaseConnection) -> anyhow::Result<InteractionHandler> {
    log::info!("Initializing Discord Module");

    let app_id: Snowflake = std::env::var("DISCORD_APP_ID").expect("DISCORD_APP_ID not set").parse().expect("DISCORD_APP_ID is not a valid Snowflake");
    let public_key = std::env::var("DISCORD_PUBLIC_KEY").expect("DISCORD_PUBLIC_KEY not set");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let owner_id: Snowflake = std::env::var("DISCORD_BOT_OWNER_ID").expect("DISCORD_BOT_OWNER_ID not set").parse().expect("DISCORD_OWNER_ID is not a valid Snowflake");
    let webhook_url = std::env::var("DISCORD_WEBHOOK_URL").ok();
    let moderator_roles = std::env::var("DISCORD_MODERATOR_ROLES").ok();
    let guild_id: Option<Snowflake> = std::env::var("DISCORD_GUILD_ID").ok().map(|id| id.parse().expect("DISCORD_GUILD_ID is not a valid Snowflake"));

    unsafe {
        OWNER_ID = owner_id;
        GUILD_ID = guild_id.unwrap_or(0);
    }

    if let Some(roles) = moderator_roles {
        unsafe {
            MODERATOR_ROLES = roles.split(",").map(|r| r.parse().expect("DISCORD_MODERATOR_ROLES contains an invalid Snowflake")).collect();
        }
    }

    let mut handler = InteractionHandler::new(app_id, public_key, Some(&token));
    handler.data.insert(db);

    if let Some(url) = webhook_url {
        handler.data.insert(Webhook::new(url));
    }

    handler.add_global_command("reload", reload_commands);
    commands::register_commands(&mut handler);
    update_global_commands(&mut handler, app_id).await?;

    Ok(handler)
}

#[defer]
#[slash_command]
async fn reload_commands(handler: &mut InteractionHandler, ctx: Context) -> InteractionResponse {

    let owner = unsafe { OWNER_ID };

    match ctx.author_id {
        Some(id) => {
            if id != owner {
                return ctx.respond()
                    .content("Only the application owner can use this command")
                    .is_ephemeral(true)
                    .finish();
            }
        }
        None => {
            return ctx.respond()
                .content("Cannot use this command without being a user")
                .is_ephemeral(true)
                .finish();
        }
    }


    log::info!("Reloading commands");

    match update_global_commands(handler, ctx.interaction.application_id.unwrap()).await {
        Ok(_) => {
            ctx.respond()
                .content("Reloaded commands")
                .is_ephemeral(true)
                .finish()
        }
        Err(e) => {
            log::error!("Failed to reload commands: {}", e);
            ctx.respond()
                .content("Failed to reload commands")
                .is_ephemeral(true)
                .finish()
        }
    }
}



