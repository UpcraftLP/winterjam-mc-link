use rusty_interaction::types::interaction::WebhookMessage;

#[derive(Clone, Debug)]
pub(crate) struct Webhook {
    url: String,
}

impl Webhook {
    pub fn new(url: String) -> Self {
        Self {
            url,
        }
    }

    pub async fn send(&self, message: WebhookMessage) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        match client.post(&self.url).json(&message).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    anyhow::bail!("Failed to send webhook - {}: {:?}", response.status(), response.text().await?);
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to send webhook: {}", e);
            }
        }
        Ok(())
    }
}