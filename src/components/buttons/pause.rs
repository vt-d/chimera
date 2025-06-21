use crate::command_handler::StateExt;
use std::sync::Arc;
use twilight_model::{
    application::interaction::Interaction,
    channel::message::{
        Component,
        component::{ActionRow, Button},
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub async fn pause_button_handler(
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

    let pause_resume = !player_data.paused;
    player.set_pause(pause_resume).await?;

    interaction_client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .components(vec![action_menu(pause_resume)])
                        .build(),
                ),
            },
        )
        .await?;

    Ok(())
}

pub fn action_menu(pause_resume: bool) -> Component {
    if pause_resume {
        return Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    label: Some("▶️ Resume".to_string()),
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
        });
    }

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
