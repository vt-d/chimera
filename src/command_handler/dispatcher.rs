use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::Message,
};

use crate::command_handler::definition::GlobalState;
use crate::commands::COMMANDS;

pub async fn slash_handler(
    interaction: Interaction,
    data: CommandData,
    state: GlobalState,
) -> anyhow::Result<()> {
    for cmd_def in COMMANDS.iter() {
        if cmd_def.name == data.name.as_str() {
            return (cmd_def.slash_executor)(state, interaction, data).await;
        }
    }
    tracing::warn!("Unknown slash command: {}", data.name);
    Ok(())
}

pub async fn prefix_handler(
    message: Message,
    configured_prefix: &str,
    state: GlobalState,
) -> anyhow::Result<()> {
    if message.author.bot {
        return Ok(());
    }

    if let Some(parsed_command) = crate::prefix_parser::parse(&message.content, configured_prefix) {
        let command_name = parsed_command.command;
        let arguments = parsed_command.arguments();
        let prefix_string = configured_prefix.to_string();

        for cmd_def in COMMANDS.iter() {
            if cmd_def.name == command_name || cmd_def.aliases.contains(&command_name) {
                return (cmd_def.prefix_executor)(state, &message, arguments, prefix_string).await;
            }
        }
        tracing::warn!("Unknown prefix command: {}", command_name);
    }
    Ok(())
}
