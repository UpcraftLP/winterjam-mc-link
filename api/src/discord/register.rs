use rusty_interaction::Builder;
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::application::{ApplicationCommand, ApplicationCommandOption, ApplicationCommandOptionType, SlashCommandDefinitionBuilder};
use rusty_interaction::types::Snowflake;

const BASE_URL: &str = rusty_interaction::BASE_URL;

pub(crate) async fn update_global_commands(handler: &mut InteractionHandler, app_id: Snowflake) -> anyhow::Result<()> {
    let commands: Vec<ApplicationCommand> = vec![
        SlashCommandDefinitionBuilder::default()
            .name("reload")
            .description("Reload the commands")
            .default_permission(false)
            .build().unwrap(),
        SlashCommandDefinitionBuilder::default()
            .name("whitelist")
            .description("Add a user to the whitelist")
            .add_option(ApplicationCommandOption::default()
                            .name("username")
                            .option_type(&ApplicationCommandOptionType::String)
                            .required(&true)
                            .description("Your Minecraft username"),
            )
            .build().unwrap(),
    ];

    let url = format!("{BASE_URL}/applications/{app_id}/commands");
    let response = handler.client().clone().put(url).json(&commands).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to update global commands: {:?}", response.text().await?);
    }

    Ok(())
}