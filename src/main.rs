#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mc_link_api::start().await
}
