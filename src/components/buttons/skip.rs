use crate::command_handler::{CommandResponseBuilder, StateExt};
use std::sync::Arc;
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub async fn skip_button_handler(
    state: Arc<crate::state::State>,
    interaction: Interaction,
) -> anyhow::Result<()> {
    let http = state.http.clone();
    let interaction_client = http.interaction(interaction.application_id);
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| anyhow::anyhow!("Interaction must be in a guild to skip the track"))?;

    let player = state
        .lavalink()
        .get_player_context(guild_id)
        .ok_or_else(|| anyhow::anyhow!("No player found for guild: {}", guild_id))?;
    let player_data = player.get_player().await?;

    let message = interaction
        .message
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("message not found"))?;

    let track = player_data
        .track
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No track is currently playing in guild: {}", guild_id))?;

    player.skip()?;

    interaction_client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content(format!(
                            "️⏩ Skipped {} to the next track.",
                            track.info.title
                        ))
                        .embeds(Vec::new())
                        .components(Vec::new())
                        .build(),
                ),
            },
        )
        .await?;

        let response = CommandResponseBuilder::new()
            .content(format!(
                "️⏩ Skipped {} to the next track.",
                track.info.title
            ))
            .build();
    
    state.http
        .create_message(message.channel_id)
        .content(&response.content)
        .embeds(&response.embeds)
        .components(&response.components)
        .await?;

    Ok(())
}
