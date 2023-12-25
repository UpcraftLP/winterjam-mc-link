use serde::Deserialize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let expected_version = option_env!("VERSION").unwrap_or("unknown");

    let resp = reqwest::get("http://localhost:3000/_health").await?
        .json::<HealthCheckResponse>().await?;

    if resp.status != "ok" {
        anyhow::bail!("Health check failed: {}", resp.status);
    }

    if resp.version != expected_version {
        anyhow::bail!("Version mismatch: expected {}, got {}", expected_version, resp.version);
    }

    Ok(())
}

#[derive(Deserialize)]
struct HealthCheckResponse {
    status: String,
    version: String,
}