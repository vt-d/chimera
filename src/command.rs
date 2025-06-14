pub mod ping;
pub mod music; // allow for extension someday; modulizing this

use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{Interaction, application_command::CommandData};
use twilight_model::channel::Message;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

use crate::prefix_parser::Arguments;
use crate::State;

pub struct PrefixContext<'a> {
    pub message_id: twilight_model::id::Id<twilight_model::id::marker::MessageMarker>,
    pub channel_id: twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
    pub message: &'a Message,
    pub parsed: Arguments<'a>,
    pub prefix: String,
    pub http_client: Arc<HttpClient>,
}

impl<'a> PrefixContext<'a> {
    pub async fn reply(&self, content: impl Into<String> + Send) -> anyhow::Result<()> {
        let reply_content = content.into();
        self.http_client
            .create_message(self.channel_id)
            .content(&reply_content)
            .reply(self.message_id)
            .await?;
        Ok(())
    }
}

pub struct SlashContext {
    pub interaction: Interaction,
    pub data: CommandData,
    pub http_client: Arc<HttpClient>,
}

impl SlashContext {
    pub async fn reply(&self, content: impl Into<String> + Send) -> anyhow::Result<()> {
        let reply_content = content.into();
        let interaction = &self.interaction;

        self.http_client
            .interaction(interaction.application_id)
            .create_response(
                interaction.id,
                &interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        content: Some(reply_content),
                        ..Default::default()
                    }),
                },
            )
            .await?;
        Ok(())
    }
}

pub enum CommandContext<'ctx> {
    Prefix(Box<PrefixContext<'ctx>>),
    Slash(Box<SlashContext>),
}

impl<'ctx> CommandContext<'ctx> {
    pub async fn reply(&self, content: impl Into<String> + Send) -> anyhow::Result<()> {
        match self {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.reply(content).await,
            CommandContext::Slash(slash_ctx) => slash_ctx.reply(content).await,
        }
    }
}

#[async_trait::async_trait]
pub trait Command<S>: CreateCommand
where
    S: HasHttpClient + Clone + Send + Sync + 'static + Sized,
{
    async fn execute<'ctx>(state: S, cmd_ctx: CommandContext<'ctx>) -> anyhow::Result<()>;

    async fn execute_prefix_command<'msg_lifetime>(
        state: S,
        message_ref: &'msg_lifetime Message,
        arguments: Arguments<'msg_lifetime>,
        prefix_str: String,
    ) -> anyhow::Result<()> {
        let prefix_ctx = PrefixContext {
            message_id: message_ref.id,
            channel_id: message_ref.channel_id,
            parsed: arguments,
            prefix: prefix_str,
            http_client: state.http_client(),
            message: message_ref,   
        };
        Self::execute(state, CommandContext::Prefix(Box::new(prefix_ctx)))
            .await
    }

    async fn execute_slash_command(
        state: S,
        interaction: Interaction,
        data: CommandData,
    ) -> anyhow::Result<()> {
        let slash_ctx = SlashContext {
            interaction,
            data,
            http_client: state.http_client(),
        };
        Self::execute(state, CommandContext::Slash(Box::new(slash_ctx)))
            .await
    }
}

pub async fn slash_handler(
    interaction: Interaction,
    data: CommandData,
    client: Arc<HttpClient>,
) -> anyhow::Result<()> {
    match &*data.name {
        "ping" => {
            ping::PingCommand::execute_slash_command(State { http: client.clone() }, interaction, data).await?;
        },
        name => {
            tracing::warn!("Unknown slash command: {}", name);
        }
    }
    Ok(())
}

pub async fn prefix_handler(
    message: Message,
    client: Arc<HttpClient>,
    configured_prefix: &str,
) -> anyhow::Result<()> {
    if message.author.bot {
        return Ok(());
    }

    match crate::prefix_parser::parse(&message.content, configured_prefix) {
        Some(parsed_command) => {
            let command_name = parsed_command.command;
            let arguments = parsed_command.arguments();
            let state = State { http: client.clone() };
            let prefix_string = configured_prefix.to_string();

            match command_name {
                "ping" => {
                    ping::PingCommand::execute_prefix_command(state, &message, arguments, prefix_string)
                        .await?;
                },
                "play" => {
                    music::PlayCommand::execute_prefix_command(state, &message, arguments, prefix_string)
                        .await?;
                },
                name => {
                    tracing::debug!("Unknown prefix command: {} from user: {}", name, message.author.name);
                }
            }
        }
        None => {
        }
    }
    Ok(())
}

pub trait HasHttpClient {
    fn http_client(&self) -> Arc<HttpClient>;
}

pub trait StateExt: HasHttpClient {}