use super::check_voice_state;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt,
};
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "volume", desc = "Change the volume of the player.")]
pub struct VolumeCommand {
    #[allow(unused)]
    #[command(desc = "Volume level (0-150)")]
    volume: i64,
}

#[async_trait]
impl Command<GlobalState> for VolumeCommand {
    async fn execute<'ctx>(state: GlobalState, mut cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?;

        let volume: i64 = cmd_ctx.get_arg("volume").ok_or_else(|| {
            anyhow::anyhow!("Volume argument is required and must be a number between 0 and 150.")
        })?;

        if !(0..=150).contains(&volume) {
            anyhow::bail!("Volume must be between 0 and 150.");
        }

        check_voice_state(state.clone(), &cmd_ctx).await?;

        let player = state
            .lavalink()
            .get_player_context(guild_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for this guild."))?;

        player.set_volume(volume as u16).await?;

        let response = CommandResponseBuilder::new()
            .content(format!("Volume set to {}.", volume))
            .build();

        cmd_ctx.reply(response).await?;
        Ok(())
    }
}
