use crate::command_handler::{Command, CommandContext, CommandResponseBuilder, GlobalState};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Check if the bot is responsive.")]
pub struct PingCommand;

#[async_trait]
impl Command<GlobalState> for PingCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let ping_lock = state.latency_ms.lock().await;
        let ping = *ping_lock;

        drop(ping_lock);

        if ping.is_none() {
            return Err(anyhow::anyhow!(
                "Latency is not available; Not enough data collected yet."
            ));
        }
        let response = CommandResponseBuilder::new()
            .content(format!("🏓 Pong! `({}ms)`", ping.unwrap()))
            .build();

        cmd_ctx.reply(response).await?;
        Ok(())
    }
}
