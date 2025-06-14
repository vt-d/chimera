pub mod music;
pub mod ping; // allow for extension someday; modulizing this

use std::sync::Arc;

use lavalink_rs::client::LavalinkClient;
use songbird::Songbird;
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{Interaction, application_command::CommandData};
use twilight_model::channel::Message;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};

use crate::match_command;
use crate::prefix_parser::Arguments;
use crate::state::State;

pub type GlobalStateInner = State;
pub type GlobalState = Arc<GlobalStateInner>;

pub struct PrefixContext<'a> {
    pub message_id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
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
    S: HasHttpClient + StateExt + Clone + Send + Sync + 'static + Sized,
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
        Self::execute(state, CommandContext::Prefix(Box::new(prefix_ctx))).await
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
        Self::execute(state, CommandContext::Slash(Box::new(slash_ctx))).await
    }
}

pub async fn slash_handler(
    interaction: Interaction,
    data: CommandData,
    client: Arc<HttpClient>,
    state: GlobalState,
) -> anyhow::Result<()> {
    match_command!(&*data.name, state.clone(), interaction, data, {
        "ping" => ping::PingCommand,
    });
    Ok(())
}

async fn run_prefix_command<'msg_lifetime, C>(
    state: GlobalState,
    message: &'msg_lifetime Message,
    arguments: Arguments<'msg_lifetime>,
    prefix_string: String,
) -> anyhow::Result<()>
where
    C: Command<GlobalState> + Send,
{
    C::execute_prefix_command(state.clone(), message, arguments, prefix_string).await
}

pub async fn prefix_handler(
    message: Message,
    client: Arc<HttpClient>,
    configured_prefix: &str,
    state: Arc<GlobalStateInner>,
) -> anyhow::Result<()> {
    if message.author.bot {
        return Ok(());
    }

    if let Some(parsed_command) = crate::prefix_parser::parse(&message.content, configured_prefix) {
        let command_name = parsed_command.command;
        let arguments = parsed_command.arguments();
        let prefix_string = configured_prefix.to_string();

        match_command!(command_name, state, &message, arguments, prefix_string, {
            "ping" => ping::PingCommand,
            "play" => music::PlayCommand,
        });
    }
    Ok(())
}

pub trait HasHttpClient {
    fn http_client(&self) -> Arc<HttpClient>;
}

pub trait StateExt: HasHttpClient {
    fn lavalink(&self) -> Arc<LavalinkClient>;
    fn songbird(&self) -> Arc<Songbird>;
}
