use crate::command::{Command, CommandContext, GlobalState, HasHttpClient};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Check if the bot is responsive.")]
pub struct PingCommand;

#[async_trait]
impl Command<GlobalState> for PingCommand {
    async fn execute<'ctx>(_state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        cmd_ctx.reply("Pong!").await?;
        Ok(())
    }
}
