use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub line: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LyricsApiResponse {
    pub lines: Vec<LyricLine>,
}

pub async fn get_lyrics(
    address: &str,
    session_id: &str,
    guild_id: &str,
    client: &Client,
    token: &str,
) -> Result<String> {
    let url = format!(
        "{}/v4/sessions/{}/players/{}/track/lyrics?skipTrackSource=false",
        address, session_id, guild_id
    );
    tracing::info!("Fetching lyrics from: {}", url);
    let response = client
        .get(&url)
        .header("Authorization", token)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        tracing::error!(
            "Failed to fetch lyrics. Status: {}, Body: {}",
            status,
            error_text
        );
        return Err(anyhow!(
            "Failed to fetch lyrics. Status: {}, Body: {}",
            status,
            error_text
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| anyhow!("Failed to get lyrics response text: {}", e))?;
    let lyrics_response: LyricsApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| anyhow!("Failed to parse lyrics response: {}", e))?;
    let lyrics_text = lyrics_response
        .lines
        .into_iter()
        .map(|line| line.line)
        .collect::<Vec<String>>()
        .join("\n");

    Ok(lyrics_text)
}
