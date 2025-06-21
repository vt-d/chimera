use crate::command_handler::StateExt;
use crate::utils::lyrics::get_lyrics;
use anyhow::anyhow;
use std::sync::Arc;
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub async fn lyrics_button_handler(
    state: Arc<crate::state::State>,
    interaction: Interaction,
) -> anyhow::Result<()> {
    let http = state.http.clone();
    let interaction_client = http.interaction(interaction.application_id);
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| anyhow!("Interaction must be in a guild to skip the track"))?;
    let message = interaction
        .message
        .as_ref()
        .ok_or_else(|| anyhow!("Interaction must have a message to skip the track"))?;

    let node = state.lavalink().get_node_for_guild(guild_id).await;
    let address = &node.http.rest_address_versionless;
    let session_id = node.session_id.load();
    let token = node
        .http
        .headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Failed to get Authorization header from node"))?;

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

    interaction_client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content(String::new())
                        .build(),
                ),
            },
        )
        .await?;

    state
        .http
        .create_message(message.channel_id)
        .embeds(&[embed])
        .reply(message.id)
        .await?;

    Ok(())
}
