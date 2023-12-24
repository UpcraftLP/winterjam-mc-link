use std::fmt::Display;
use anyhow::Context;
use serde::Deserialize;
use uuid::Uuid;

pub(crate) async fn resolve_username(username: &impl Display) -> anyhow::Result<Option<MojangResponse>> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{username}");

    let response = reqwest::get(url).await
        .context("Failed to resolve username")?;

    if !response.status().is_success() {
        if response.status() != reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Failed to resolve username: {:?}", response.text().await);
        }
        return Ok(None);
    }

    let value = response.json::<MojangResponse>().await
        .context("Failed to parse response")?;

    Ok(Some(value))
}

#[derive(Deserialize, Debug)]
pub(crate) struct MojangResponse {
    pub(crate) id: Uuid,
    pub(crate) name: String,
}
