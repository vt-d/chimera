use crate::command_handler::{Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt};
use crate::utils::lyrics::get_lyrics;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "lyrics", desc = "Get the lyrics for the current song.")]
pub struct LyricsCommand;

#[async_trait]
impl Command<GlobalState> for LyricsCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = cmd_ctx
            .guild_id()
            .ok_or_else(|| anyhow!("Interaction must be in a guild"))?;

        let node = state.lavalink().get_node_for_guild(guild_id).await;
        let address = &node.http.rest_address_versionless;
        let session_id = node.session_id.load();
        let token = node
            .http
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("Failed to get Authorization header from node"))?;

        let guild_id_str = guild_id.to_string();
        let lyrics = get_lyrics(
            address,
            &session_id,
            &guild_id_str,
            &state.reqwest,
            token,
        )
        .await?;

        let embed = twilight_util::builder::embed::EmbedBuilder::new()
            .title("🎶 Lyrics")
            .description(lyrics)
            .color(0x1DB954)
            .build();

        cmd_ctx
            .reply(CommandResponseBuilder::new().embed(embed).build())
            .await?;

        Ok(())
    }
}