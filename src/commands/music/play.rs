use std::collections::VecDeque;

use super::join;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, HasHttpClient,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use lavalink_rs::prelude::{SearchEngines, TrackInQueue, TrackLoadData};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::Message,
    id::{Id, marker::GuildMarker},
};

#[derive(CreateCommand)]
#[command(name = "play", desc = "Play a song from YouTube or other sources.")]
pub struct PlayCommand {
    #[command(desc = "The song to play")]
    pub song: String,
}

#[async_trait]
impl Command<GlobalState> for PlayCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let song_query = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.parsed.remainder().to_string(),
            CommandContext::Slash(slash_ctx) => slash_ctx
                .data
                .options
                .iter()
                .find(|opt| opt.name == "song")
                .and_then(|opt| match &opt.value {
                    CommandOptionValue::String(s_val) => Some(s_val.clone()),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("Song query is missing or not a string."))?,
        };

        let author = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => &prefix_ctx.message.author,
            CommandContext::Slash(slash_ctx) => slash_ctx
                .interaction
                .author()
                .ok_or_else(|| anyhow!("Interaction is missing author information."))?,
        };

        let guild_id: Id<GuildMarker> = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow!("This command must be used in a guild."))?;

        let voice_state = state
            .voice_states
            .get(&author.id)
            .ok_or_else(|| anyhow!("You must be in a voice channel to use this command."))?;

        let msg = join(
            state.clone(),
            &cmd_ctx,
            voice_state.clone(),
            None,
            state.http_client(),
        )
        .await?;

        let lava_client = state.lavalink.clone();

        let player = lava_client
            .get_player_context(guild_id)
            .ok_or_else(|| anyhow!("Player context not found. Is the bot in a voice channel?"))?;

        let query_term = if song_query.starts_with("http") {
            song_query.clone()
        } else if (song_query.contains(':') && song_query.split(':').count() == 2)
            || song_query.contains(" - ")
        {
            song_query
        } else {
            SearchEngines::Spotify.to_query(&song_query)?
        };

        let loaded_tracks_response = lava_client.load_tracks(guild_id, &query_term).await?;

        let (mut tracks_to_queue, opt_playlist_info) = match loaded_tracks_response.data {
            Some(TrackLoadData::Track(track)) => (vec![track.into()], None),
            Some(TrackLoadData::Search(search_results)) => {
                let track = search_results
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("No tracks found from search."))?;
                (vec![track.into()], None)
            }
            Some(TrackLoadData::Playlist(playlist)) => {
                let p_info = Some(playlist.info);
                let p_tracks = playlist
                    .tracks
                    .into_iter()
                    .map(TrackInQueue::from)
                    .collect();
                (p_tracks, p_info)
            }
            Some(TrackLoadData::Error(e)) => {
                let response_builder = CommandResponseBuilder::new()
                    .content(format!("Error loading tracks: {}", e.message));
                reply_to_join(&state, &cmd_ctx, &msg, response_builder).await?;
                return Ok(());
            }
            None => {
                let response_builder = CommandResponseBuilder::new()
                    .content("Failed to load tracks: received no data from Lavalink.");
                reply_to_join(&state, &cmd_ctx, &msg, response_builder).await?;
                return Ok(());
            }
        };

        if tracks_to_queue.is_empty() {
            let response_builder =
                CommandResponseBuilder::new().content("No tracks were loaded to queue.");
            reply_to_join(&state, &cmd_ctx, &msg, response_builder).await?;
            return Ok(());
        }

        tracks_to_queue.iter_mut().for_each(|track_in_queue| {
            track_in_queue.track.user_data = Some(serde_json::json!({ "requester_id": author.id }));
        });

        let reply_message = if let Some(p_info) = &opt_playlist_info {
            format!(
                "`＋`Queued playlist: [{}] ({} tracks)",
                p_info.name,
                tracks_to_queue.len()
            )
        } else if let Some(uri) = tracks_to_queue[0].track.info.uri.as_ref() {
            format!(
                "`＋` Queued [`{}`](<{}>)",
                tracks_to_queue[0].track.info.title, uri
            )
        } else {
            format!("`＋` Queued: `{}`", tracks_to_queue[0].track.info.title)
        };

        let queue = player.get_queue();
        queue.append(VecDeque::from(tracks_to_queue))?;

        let response_builder = CommandResponseBuilder::new().content(reply_message);
        reply_to_join(&state, &cmd_ctx, &msg, response_builder).await?;

            if let Ok(player_data) = player.get_player().await {
        if player_data.track.is_none() && queue.get_track(0).await.is_ok_and(|x| x.is_some()) {
            player.skip()?;
        }
    }

        Ok(())
    }
}

pub async fn reply_to_join(
    state: &GlobalState,
    ctx: &CommandContext<'_>,
    msg: &Option<Message>,
    response_builder: CommandResponseBuilder,
) -> Result<()> {
    let response = response_builder.build();

    if let Some(msg) = msg {
        let http_client = state.http_client();
        let mut create_message = http_client.create_message(msg.channel_id).reply(msg.id);

        if !response.content.is_empty() {
            create_message = create_message.content(&response.content);
        }
        if !response.embeds.is_empty() {
            create_message = create_message.embeds(&response.embeds);
        }
        if !response.components.is_empty() {
            create_message = create_message.components(&response.components);
        }

        create_message
            .await
            .map_err(|e| anyhow!("Failed to send reply to join message: {}", e))?;
    } else {
        ctx.reply(response)
            .await
            .map_err(|e| anyhow!("Failed to send response to command context: {}", e))?;
    }
    Ok(())
}
