use super::check_voice_state;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt,
};
use anyhow::Result;
use async_trait::async_trait;
use lavalink_rs::model::track::TrackData;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::{
    Component, Embed,
    component::{ActionRow, Button},
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

#[derive(CommandModel, CreateCommand)]
#[command(name = "now_playing", desc = "Show the currently playing song.")]
pub struct NowPlayingCommand;

#[async_trait]
impl Command<GlobalState> for NowPlayingCommand {
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
        let player_data = player.get_player().await?;
        let track = player_data
            .track
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No track is currently playing."))?;
        let volume = player_data.volume;
        let position = player_data.state.position / 1000; // convert to seconds
        let embed = build_now_playing_embed(track, volume, position).await?;
        let component = action_menu().await;
        let response = CommandResponseBuilder::new()
            .embed(embed.clone())
            .component(component)
            .build();

        cmd_ctx.reply(response).await?;

        Ok(())
    }
}

pub async fn build_now_playing_embed(
    track: &TrackData,
    volume: u16,
    position: u64,
) -> Result<Embed> {
    let finishing_time =
        chrono::Utc::now().timestamp() as u64 + position + (track.info.length / 1000);
    let parsed_duration = humantime::format_duration(std::time::Duration::from_secs(position));
    let embed = EmbedBuilder::new()
        .title("🎶 Now Playing")
        .description(format!(
            "**{}** by **{}**",
            track.info.title, track.info.author
        ))
        .color(0x1DB954)
        .thumbnail(ImageSource::url(track.info.artwork_url.clone().unwrap_or_default()).unwrap())
        .url(track.info.uri.clone().unwrap_or_default())
        .field(
            EmbedFieldBuilder::new(
                "Duration",
                format!(
                    "{} / {}",
                    parsed_duration,
                    humantime::format_duration(std::time::Duration::from_secs(
                        track.info.length / 1000
                    ))
                ),
            )
            .inline()
            .build(),
        )
        .field(EmbedFieldBuilder::new("Finished in", format!("<t:{}:R>", finishing_time)).inline())
        .field(EmbedFieldBuilder::new("Volume", format!("{}%", volume)).inline())
        .build();

    Ok(embed)
}

pub async fn action_menu() -> Component {
    Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                label: Some("⏸️ Pause".to_string()),
                custom_id: Some("pause".to_string()),
                style: twilight_model::channel::message::component::ButtonStyle::Secondary,
                emoji: None,
                disabled: false,
                url: None,
                sku_id: None,
            }),
            Component::Button(Button {
                label: Some("🎤 Lyrics".to_string()),
                custom_id: Some("lyrics".to_string()),
                style: twilight_model::channel::message::component::ButtonStyle::Secondary,
                emoji: None,
                disabled: false,
                url: None,
                sku_id: None,
            }),
            Component::Button(Button {
                label: Some("⏩ Skip".to_string()),
                custom_id: Some("skip".to_string()),
                style: twilight_model::channel::message::component::ButtonStyle::Danger,
                emoji: None,
                disabled: false,
                url: None,
                sku_id: None,
            }),
        ],
    })
}
