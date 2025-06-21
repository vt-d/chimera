mod help;
mod jump;
mod lyrics;
mod now_playing;
mod play;
mod queue;
mod skip;
mod stop;
mod volume;

pub use help::HelpCommand;
pub use jump::JumpCommand;
pub use lyrics::LyricsCommand;
pub use now_playing::NowPlayingCommand;
pub use play::PlayCommand;
pub use queue::QueueCommand;
pub use skip::SkipCommand;
pub use stop::StopCommand;
pub use volume::VolumeCommand;


use std::sync::Arc;

use lavalink_rs::model::player::ConnectionInfo;
use twilight_mention::Mention;
use twilight_model::channel::Message;

use crate::command_handler::{CommandResponseBuilder, StateExt};

use songbird::ConnectionInfo as SongbirdConnectionInfo;

fn convert_connection_info(connection_info: SongbirdConnectionInfo) -> ConnectionInfo {
    ConnectionInfo {
        endpoint: connection_info.endpoint,
        token: connection_info.token,
        session_id: connection_info.session_id,
    }
}

pub async fn join(
    state: Arc<crate::state::State>,
    ctx: &crate::command_handler::CommandContext<'_>,
    channel_id: twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
    guild_id: twilight_model::id::Id<twilight_model::id::marker::GuildMarker>,
    http_client: Arc<twilight_http::Client>,
) -> anyhow::Result<Option<Message>> {
    if state.songbird().get(guild_id).is_some() {
        return Ok(None);
    }

    let (connection_info, _) = state
        .songbird()
        .join_gateway(guild_id, channel_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to join voice channel, {}", e))?;

    state
        .lavalink()
        .create_player_context_with_data::<(
            twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
            std::sync::Arc<twilight_http::Client>,
        )>(
            guild_id,
            convert_connection_info(connection_info),
            Arc::new((channel_id, http_client)),
        )
        .await?;

    let response = CommandResponseBuilder::new()
        .content(format!("🎙️ Joined {}", channel_id.mention()))
        .build();
    let msg = ctx.reply(response).await?;

    Ok(Some(msg))
}

pub async fn leave(
    state: Arc<crate::state::State>,
    guild_id: twilight_model::id::Id<twilight_model::id::marker::GuildMarker>,
) -> anyhow::Result<()> {
    state.songbird().remove(guild_id).await?;
    state.lavalink().delete_player(guild_id).await?;
    Ok(())
}

pub async fn check_voice_state(
    state: Arc<crate::state::State>,
    ctx: &crate::command_handler::CommandContext<'_>,
) -> anyhow::Result<()> {
    let author = match &ctx {
        crate::command_handler::CommandContext::Prefix(prefix_ctx) => &prefix_ctx.message.author,
        crate::command_handler::CommandContext::Slash(slash_ctx) => slash_ctx
            .interaction
            .author()
            .ok_or_else(|| anyhow::anyhow!("Interaction is missing author information."))?,
    };
    let guild_id =
        match &ctx {
            crate::command_handler::CommandContext::Prefix(prefix_ctx) => prefix_ctx
                .message
                .guild_id
                .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?,
            crate::command_handler::CommandContext::Slash(slash_ctx) => slash_ctx
                .interaction
                .guild_id
                .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?,
        };

    state
        .cache
        .voice_state(author.id, guild_id)
        .ok_or_else(|| anyhow::anyhow!("You must be in a voice channel to use this command."))?;

    Ok(())
}
