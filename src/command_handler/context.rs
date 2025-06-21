use std::str::FromStr;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::{
        Interaction,
        application_command::{CommandData, CommandOptionValue},
    },
    channel::Message,
    id::{
        Id,
        marker::{ChannelMarker, GuildMarker, MessageMarker},
    },
};

use crate::command_handler::response::CommandResponse;
use crate::prefix_parser::Arguments;

pub trait FromCommandOptionValue: Sized {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self>;
}

impl FromCommandOptionValue for String {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::String(s) = value {
            Some(s.clone())
        } else {
            None
        }
    }
}

impl FromCommandOptionValue for i64 {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Integer(i) = value {
            Some(*i)
        } else {
            None
        }
    }
}

impl FromCommandOptionValue for u64 {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Integer(i) => (*i).try_into().ok(),
            CommandOptionValue::User(id) => Some(id.get()),
            CommandOptionValue::Channel(id) => Some(id.get()),
            CommandOptionValue::Role(id) => Some(id.get()),
            CommandOptionValue::Mentionable(id) => Some(id.get()),
            CommandOptionValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for usize {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Integer(i) = value {
            (*i).try_into().ok()
        } else {
            None
        }
    }
}

impl FromCommandOptionValue for bool {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Boolean(b) = value {
            Some(*b)
        } else {
            None
        }
    }
}

impl FromCommandOptionValue for f64 {
    fn from_option_value(value: &CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Number(n) = value {
            Some(*n)
        } else {
            None
        }
    }
}

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

    pub fn get_arg<T>(&mut self, name: &str) -> Option<T>
    where
        T: FromStr + FromCommandOptionValue,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        match self {
            CommandContext::Prefix(prefix_ctx) => {
                prefix_ctx.as_mut().parsed.next().and_then(|s| {
                    s.parse::<T>()
                        .map_err(|e| {
                            tracing::debug!("Failed to parse prefix arg '{}' as T: {:?}", s, e);
                        })
                        .ok()
                })
            }
            CommandContext::Slash(slash_ctx) => slash_ctx
                .as_ref()
                .data
                .options
                .iter()
                .find(|opt| opt.name == name)
                .and_then(|opt_data| {
                    T::from_option_value(&opt_data.value).or_else(|| {
                        let as_string = match &opt_data.value {
                            CommandOptionValue::String(s) => Some(s.clone()),
                            CommandOptionValue::Integer(i) => Some(i.to_string()),
                            CommandOptionValue::Boolean(b) => Some(b.to_string()),
                            CommandOptionValue::Number(n) => Some(n.to_string()),
                            CommandOptionValue::User(id) => Some(id.to_string()),
                            CommandOptionValue::Channel(id) => Some(id.to_string()),
                            CommandOptionValue::Role(id) => Some(id.to_string()),
                            CommandOptionValue::Mentionable(id) => Some(id.to_string()),
                            _ => None,
                        };

                        as_string.and_then(|s| {
                            s.parse::<T>()
                                .map_err(|e| {
                                    tracing::debug!(
                                        "Failed to parse slash arg '{}' value '{:?}' as T via FromStr fallback: {:?}",
                                        name,
                                        opt_data.value,
                                        e
                                    );
                                })
                                .ok()
                        })
                    })
                }),
        }
    }

    pub fn get_remainder_arg(&mut self, name: &str) -> Option<String> {
        match self {
            CommandContext::Prefix(prefix_ctx) => {
                let remainder = prefix_ctx.parsed.remainder().to_string();
                while prefix_ctx.parsed.next().is_some() {}
                if remainder.is_empty() {
                    None
                } else {
                    Some(remainder)
                }
            }
            CommandContext::Slash(_) => self.get_arg(name),
        }
    }

    pub fn author(&self) -> Option<&twilight_model::user::User> {
        match self {
            CommandContext::Prefix(prefix_ctx) => Some(&prefix_ctx.message.author),
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.user.as_ref(),
        }
    }

    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        match self {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
    }

    pub fn channel_id(&self) -> Option<Id<ChannelMarker>> {
        match self {
            CommandContext::Prefix(prefix_ctx) => Some(prefix_ctx.channel_id),
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.channel.as_ref().map(|channel| channel.id),
        }
    }
}
