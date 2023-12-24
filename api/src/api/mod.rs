use actix_web::{get, HttpResponse, web};
use actix_web::web::Data;
use anyhow::Context;
use reqwest::{Client, header, StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_with::*;
use uuid::Uuid;
use entity::prelude::User;
use entity::user;
use rusty_interaction::types::Snowflake;
use crate::{discord, status};

const BASE_URL: &str = rusty_interaction::BASE_URL;

#[get("/users")]
pub(crate) async fn get_users(data: Data<DatabaseConnection>, client: Data<Client>) -> HttpResponse {
    let db = data.get_ref();
    let client = client.get_ref();

    let result= User::find().all(db).await;
    if let Err(e) = result {
        log::error!("Error getting users from DB: {}", e);
        return status::err_server("Error getting users from DB");
    }

    let mut users: Vec<UserData> = Vec::new();
    for u in result.unwrap() {
        let user_data = get_user_data(u.discord_snowflake as Snowflake, u.minecraft_uuid, client).await;
        if let Err(e) = user_data {
            log::error!("Error getting user from Discord: {}", e);
            return status::err_server("Error getting user from Discord");
        }
        users.push(user_data.unwrap());
    }

    return HttpResponse::Ok().json(GetUsersResponse {
        data: Some(users)
    });
}

#[get("/users/{uuid}")]
pub(crate) async fn get_user(info: web::Path<Uuid>, data: Data<DatabaseConnection>, client: Data<Client>) -> HttpResponse {
    let db = data.get_ref();
    let client = client.get_ref();
    let uuid = info.into_inner();

    let result = User::find().filter(user::Column::MinecraftUuid.eq(uuid)).one(db).await;
    if let Err(e) = result {
        log::error!("Error getting user from DB: {}", e);
        return status::err_server("Error getting user from DB");
    }

    let user = result.unwrap();
    if user.is_none() {
        return status::err_not_found();
    }

    let user = user.unwrap();
    let user_data = get_user_data(user.discord_snowflake as Snowflake, user.minecraft_uuid, client).await;
    if let Err(e) = user_data {
        log::error!("Error getting user from Discord: {}", e);
        return status::err_server("Error getting user from Discord");
    }

    return HttpResponse::Ok().json(user_data.unwrap());
}

async fn get_user_data(snowflake: Snowflake, uuid: Uuid, client: &Client) -> anyhow::Result<UserData> {
    let moderators = unsafe { &discord::MODERATOR_ROLES };
    let guild_id = unsafe { discord::GUILD_ID };

    let response = client.get(format!("{BASE_URL}/guilds/{guild_id}/members/{snowflake}")).header(header::ACCEPT, "application/json").send().await
        .context("Failed to get discord user info")?;
    if !response.status().is_success() {

        if response.status() != StatusCode::NOT_FOUND {
            anyhow::bail!("Error getting user from Discord - {}: {}", response.status(), response.text().await?);
        }

        return Ok(UserData {
            uuid,
            snowflake,
            ..Default::default()
        });
    }

    let member = response.json::<GuildMember>().await.context("unable to parse guild member json response")?;
    let operator = member.roles.iter().map(|s| s.parse::<Snowflake>().unwrap()).any(|r| moderators.contains(&r));

    Ok(UserData {
        access: true,
        operator,
        uuid,
        snowflake
    })
}

#[derive(Deserialize)]
struct GuildMember {
    roles: Vec<String>
}

#[derive(Serialize, Default)]
struct UserData {
    pub access: bool,
    #[serde(default)]
    pub operator: bool,
    pub uuid: Uuid,
    pub snowflake: Snowflake
}

#[derive(Serialize)]
struct GetUsersResponse {
    #[serde(default)]
    pub data: Option<Vec<UserData>>
}