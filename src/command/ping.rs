use crate::{command::{Command, CommandContext, HasHttpClient}, State};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "ping",
    desc = "Check if the bot is responsive."
)]
pub struct PingCommand;

#[async_trait]
impl Command<State> for PingCommand {
    async fn execute<'ctx>(&self, _state: State, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        cmd_ctx.reply("Pong!").await?;
        Ok(())
    }
}