use crate::{command_handler::StateExt};
use serde::Deserialize;
use std::sync::Arc;
use twilight_model::{
    application::interaction::Interaction, http::interaction::{InteractionResponse, InteractionResponseType}
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
        .ok_or_else(|| anyhow::anyhow!("Interaction must be in a guild to skip the track"))?;
    let message = interaction
        .message
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Interaction must have a message to skip the track"))?;

    let node = state
        .lavalink()
        .get_node_for_guild(guild_id)
        .await;
    let address = node.http.rest_address_versionless.clone();
    let session_id = node.session_id.load().as_ref().clone();
    let token = node.http.headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Failed to get Authorization header from node"))?;

    let lyrics = get_lyrics(address, session_id, guild_id.to_string(), state.reqwest.clone(), token).await?;
    let embed = twilight_util::builder::embed::EmbedBuilder::new()
        .title("🎶 Lyrics")
        .description(lyrics)
        .color(0x1DB954) 
        .validate()?
        .build();

        tracing::info!("here x");
    interaction_client
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::UpdateMessage,
            data: Some(InteractionResponseDataBuilder::new()
                .content(String::new())
                .build()),
        })
        .await?;

        tracing::info!("here");
    state.http.create_message(message.channel_id)
        .embeds(&[embed])
        .reply(message.id)
        .await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LyricLine {
    line: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LyricsApiResponse {
    lines: Vec<LyricLine>,
}

pub async fn get_lyrics(address: String, session_id: String, guild_id: String, client: Arc<reqwest::Client>, token: &str) -> anyhow::Result<String> {
    let url = format!("{}/v4/sessions/{}/players/{}/track/lyrics?skipTrackSource=false", address, session_id, guild_id);
    tracing::info!("Fetching lyrics from: {}", url);
    let response = client.get(&url).header("Authorization", format!("{}", token)).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        tracing::error!("Failed to fetch lyrics. Status: {}, Body: {}", status, error_text);
        return Err(anyhow::anyhow!("Failed to fetch lyrics. Status: {}, Body: {}", status, error_text));
    }

    let lyrics_response: LyricsApiResponse = serde_json::from_str(&response.text().await?)
        .map_err(|e| anyhow::anyhow!("Failed to parse lyrics response: {}", e))?;
    let lyrics_text = lyrics_response.lines.into_iter()
        .map(|line| line.line)
        .collect::<Vec<String>>()
        .join("\n");

    Ok(lyrics_text)
}