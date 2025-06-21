use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::application::interaction::InteractionData;

use crate::components;
use crate::state::State;
use once_cell::sync::Lazy;

static COMPONENT_REGISTRY: Lazy<components::ComponentRegistry> =
    Lazy::new(components::build_registry);

pub async fn process(event: Event, state: Arc<State>) -> anyhow::Result<()> {
    match event {
        Event::InteractionCreate(interaction_payload) => {
            let mut interaction = interaction_payload.0;

            match std::mem::take(&mut interaction.data) {
                Some(InteractionData::ApplicationCommand(data)) => {
                    if let Err(e) =
                        crate::command_handler::slash_handler(interaction, *data, state.clone())
                            .await
                    {
                        tracing::error!(error = ?e, "Error handling slash command");
                    }
                }
                Some(InteractionData::MessageComponent(data)) => {
                    if let Err(e) = COMPONENT_REGISTRY
                        .handle(state.clone(), interaction, *data)
                        .await
                    {
                        tracing::error!(error = ?e, "Error handling component interaction");
                    }
                }
                _ => {
                    tracing::warn!("Ignoring non-application-command interaction");
                }
            }
        }
        Event::MessageCreate(message_payload) => {
            let message = message_payload.0;
            if message.author.bot {
                return Ok(());
            }

            if let Err(e) = crate::command_handler::prefix_handler(
                message,
                &state.as_ref().config.configured_prefix,
                state.clone(),
            )
            .await
            {
                tracing::error!(error = ?e, "Error handling prefix command");
            }
        }
        _ => {}
    }
    Ok(())
}
