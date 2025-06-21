use crate::command_handler::{Command, CommandContext, CommandResponseBuilder, GlobalState};
use crate::commands::COMMANDS;
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

#[derive(CommandModel, CreateCommand)]
#[command(name = "help", desc = "Show the help menu for commands.")]
pub struct HelpCommand;

#[async_trait]
impl Command<GlobalState> for HelpCommand {
    async fn execute<'ctx>(_state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let mut embed_builder = EmbedBuilder::new()
            .title("Chimera Help")
            .description("Here is a list of all available commands.")
            .color(0x1DB954);

        for command_def in COMMANDS.iter() {
            let slash_command = (command_def.create_slash_data_fn)();
            let mut description = format!("```{}", slash_command.description);

            if !slash_command.options.is_empty() {
                description.push_str("\n\nArguments:");
                for option in slash_command.options {
                    description.push_str(&format!(
                        "\n{} ({}): {}",
                        option.name,
                        if option.required.unwrap_or(false) {
                            "required"
                        } else {
                            "optional"
                        },
                        option.description
                    ));
                }
            }
            description.push_str("```");

            embed_builder = embed_builder.field(EmbedFieldBuilder::new(
                format!("/{}", slash_command.name),
                description,
            ));
        }

        let embed = embed_builder.build();

        cmd_ctx
            .reply(CommandResponseBuilder::new().embed(embed).build())
            .await?;

        Ok(())
    }
}

