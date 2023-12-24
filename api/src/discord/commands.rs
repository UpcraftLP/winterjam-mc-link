use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use serde_with::chrono::Utc;

use entity::prelude::User;
use entity::user;
use rusty_interaction::{Builder, defer, slash_command};
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::embed::{EmbedBuilder, EmbedField, EmbedThumbnail};
use rusty_interaction::types::interaction::{Context, InteractionResponse, WebhookMessage};

use crate::mojang;

#[defer]
#[slash_command]
async fn whitelist_add(handler: &mut InteractionHandler, ctx: Context) -> InteractionResponse {
    if ctx.interaction.guild_id.is_none() {
        return ctx.respond()
            .content("This command can only be used in a server")
            .is_ephemeral(true)
            .finish();
    }

    if ctx.interaction.member.is_none() {
        return ctx.respond()
            .content("This command can only be used by a user")
            .is_ephemeral(true)
            .finish();
    }

    let discord_user = ctx.interaction.member.clone().unwrap().user;

    if let Some(data) = &ctx.interaction.data {
        let username_data = data.options.as_ref().unwrap().iter().find(|&option| option.name == "username").unwrap();
        let username = &username_data.value;

        let mojang_result = mojang::resolve_username(username).await;
        if let Err(e) = &mojang_result {
            log::error!("Failed to resolve user: {}", e);
            return ctx.respond()
                .content("Something went wrong")
                .is_ephemeral(true)
                .finish();
        }
        let response = mojang_result.unwrap();

        match response {
            None => {
                return ctx.respond()
                    .content("That user does not exist!")
                    .is_ephemeral(true)
                    .finish();
            }
            Some(response) => {
                let db = handler.data.get::<DatabaseConnection>().expect("Failed to get DB connection");

                let db_result = User::find().filter(user::Column::DiscordSnowflake.eq(discord_user.id as u64)).one(db).await;
                if let Err(e) = &db_result {
                    log::error!("Failed to get user: {}", e);
                    return ctx.respond()
                        .content("Something went wrong")
                        .is_ephemeral(true)
                        .finish();
                }
                let old: Option<user::Model> = db_result.unwrap();

                if let Some(old) = &old {
                    if old.minecraft_uuid == response.id {
                        return ctx.respond()
                            .content("That user is already whitelisted!")
                            .is_ephemeral(true)
                            .finish();
                    }
                }

                log::info!("Setting new whitelist entry for user {}: {}", discord_user.id, response.name);
                if let Some(old) = old {
                    let mut user: user::ActiveModel = old.into();

                    user.minecraft_uuid = Set(response.id);

                    user.update(db).await.expect("Failed to update user");
                } else {
                    let user = user::ActiveModel {
                        discord_snowflake: Set(discord_user.id as i64),
                        minecraft_uuid: Set(response.id),
                        ..Default::default()
                    };

                    user.insert(db).await.expect("Failed to update user");
                }

                let webhook = handler.data.get::<crate::discord::webhook::Webhook>();
                if let Some(webhook) = webhook {
                    let result = webhook.send(WebhookMessage {
                        username: Some("WinterJam".to_string()),
                        avatar_url: Some("https://winterjam.tophatcat.dev/images/util/webhook-logo.png".to_string()),
                        embeds: Some(vec![EmbedBuilder::default()
                            .title("Whitelist Update")
                            .thumbnail(EmbedThumbnail {
                                url: Some(format!("https://crafthead.net/bust/{}/128", response.id)),
                                width: Some(128),
                                height: Some(128),
                                ..Default::default()
                            })
                            .add_field(EmbedField::default()
                                .name("Discord User")
                                .value(format!("`{}` <@{}>", discord_user.id, discord_user.id))
                            )
                            .add_field(EmbedField::default()
                                .name("Minecraft Username")
                                .value(response.name)
                            )
                            .add_field(EmbedField::default()
                                .name("Minecraft UUID")
                                .value(format!("`{}`", response.id))
                            )
                            .timestamp(Utc::now())
                            .build().unwrap()
                        ]),
                        allowed_mentions: Some(Default::default()),
                        ..Default::default()
                    }
                    ).await;
                    if let Err(e) = result {
                        log::error!("Failed to send webhook: {}", e)
                    }
                }

                return ctx.respond()
                    .content(format!("Successfully added {username} to the whitelist"))
                    .is_ephemeral(true)
                    .finish();
            }
        }
    }

    // should never happen but just in case
    ctx.respond()
        .content("Something went wrong")
        .is_ephemeral(true)
        .finish()
}

pub(crate) fn register_commands(handler: &mut InteractionHandler) {
    handler.add_global_command("whitelist", whitelist_add)
}