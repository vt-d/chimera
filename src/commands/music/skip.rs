use super::check_voice_state;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt,
};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "skip", desc = "Skip the currently playing song.")]
pub struct SkipCommand;

#[async_trait]
impl Command<GlobalState> for SkipCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?;

        check_voice_state(state.clone(), &cmd_ctx).await?;

        let player = state
            .lavalink()
            .get_player_context(guild_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for this guild."))?;

        let track = player
            .get_player()
            .await?
            .track
            .ok_or_else(|| anyhow::anyhow!("No track is currently playing."))?;
        
        player.skip()?;

        let response = CommandResponseBuilder::new()
            .content(format!(
                "️⏩ Skipped {} to the next track.",
                track.info.title
            ))
            .build();

        cmd_ctx.reply(response).await?;

        Ok(())
    }
}
