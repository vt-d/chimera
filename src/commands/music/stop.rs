use super::{check_voice_state, leave};
use crate::command_handler::{Command, CommandContext, CommandResponseBuilder, GlobalState};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "stop", desc = "Stop the current music playback.")]
pub struct StopCommand;

#[async_trait]
impl Command<GlobalState> for StopCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?;

        check_voice_state(state.clone(), &cmd_ctx).await?;
        leave(state, guild_id).await?;

        let response = CommandResponseBuilder::new().content("⏹️ Stopped").build();
        cmd_ctx.reply(response).await?;

        Ok(())
    }
}
