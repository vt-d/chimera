use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::Message,
    id::{
        Id,
        marker::{ChannelMarker, MessageMarker},
    },
};

use crate::command_handler::response::CommandResponse;
use crate::prefix_parser::Arguments;

pub struct PrefixContext<'a> {
    pub message_id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub message: &'a Message,
    pub parsed: Arguments<'a>,
    pub prefix: String,
    pub http_client: Arc<HttpClient>,
}

impl<'a> PrefixContext<'a> {
    pub async fn reply(
        &self,
        response: CommandResponse,
    ) -> anyhow::Result<twilight_model::channel::Message> {
        let mut create_message = self
            .http_client
            .create_message(self.channel_id)
            .reply(self.message_id);

        if !response.content.is_empty() {
            create_message = create_message.content(&response.content);
        }
        if !response.embeds.is_empty() {
            create_message = create_message.embeds(&response.embeds);
        }
        if !response.components.is_empty() {
            create_message = create_message.components(&response.components);
        }

        let response = create_message.await?;
        let message = response.model().await?;
        Ok(message)
    }
}

pub struct SlashContext {
    pub interaction: Interaction,
    pub data: CommandData,
    pub http_client: Arc<HttpClient>,
}

impl SlashContext {
    pub async fn reply(
        &self,
        response: CommandResponse,
    ) -> anyhow::Result<twilight_model::channel::Message> {
        let interaction_client = self
            .http_client
            .interaction(self.interaction.application_id);

        let response_data = twilight_model::http::interaction::InteractionResponseData {
            content: if response.content.is_empty() {
                None
            } else {
                Some(response.content)
            },
            embeds: if response.embeds.is_empty() {
                None
            } else {
                Some(response.embeds)
            },
            components: if response.components.is_empty() {
                None
            } else {
                Some(response.components)
            },
            ..Default::default()
        };

        interaction_client
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &twilight_model::http::interaction::InteractionResponse {
                    kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
                    data: Some(response_data),
                },
            )
            .await?;

        let message_response = interaction_client.response(&self.interaction.token).await?;
        let message = message_response.model().await?;
        Ok(message)
    }
}

pub enum CommandContext<'ctx> {
    Prefix(Box<PrefixContext<'ctx>>),
    Slash(Box<SlashContext>),
}

impl<'ctx> CommandContext<'ctx> {
    pub async fn reply(
        &self,
        response: CommandResponse,
    ) -> anyhow::Result<twilight_model::channel::Message> {
        match self {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.reply(response).await,
            CommandContext::Slash(slash_ctx) => slash_ctx.reply(response).await,
        }
    }

    pub async fn reply_error(
        &self,
        error: &anyhow::Error,
        create_error_fn: impl Fn(&anyhow::Error) -> CommandResponse,
    ) -> anyhow::Result<()> {
        tracing::error!(error = ?error, "Command execution failed");
        let error_response = create_error_fn(error);
        self.reply(error_response).await?;
        Ok(())
    }

    pub fn author(&self) -> Option<&twilight_model::user::User> {
        match self {
            CommandContext::Prefix(prefix_ctx) => Some(&prefix_ctx.message.author),
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.user.as_ref(),
        }
    }
}
