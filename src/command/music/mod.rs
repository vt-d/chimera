mod play;

use std::sync::Arc;

use lavalink_rs::model::player::ConnectionInfo;
pub use play::PlayCommand;
use twilight_http::Client;

use crate::command::StateExt;

use songbird::ConnectionInfo as SongbirdConnectionInfo;

// cursed fix; TODO: change this
fn convert_connection_info(connection_info: SongbirdConnectionInfo) -> ConnectionInfo {
    ConnectionInfo {
        endpoint: connection_info.endpoint,
        token: connection_info.token,
        session_id: connection_info.session_id,
    }
}

pub async fn _join(
    state: Arc<crate::state::State>,
    channel_id: twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
    http_client: Arc<twilight_http::Client>,
    guild_id: twilight_model::id::Id<twilight_model::id::marker::GuildMarker>,
    ctx: &crate::command::CommandContext<'_>,
) -> anyhow::Result<()> {
    let (connection_info, _) = state
        .songbird()
        .join_gateway(guild_id, channel_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to join voice channel"))?;

    ctx.reply("Joined voice channel!").await?;

    state.lavalink().create_player_context_with_data::<(
        twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
        std::sync::Arc<twilight_http::Client>,
    )>(guild_id, convert_connection_info(connection_info), Arc::new((channel_id, http_client))).await?;

    Ok(())
}
