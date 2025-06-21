use std::{future::Future, pin::Pin, sync::Arc};

use lavalink_rs::client::LavalinkClient;
use songbird::Songbird;
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::Message,
};

use crate::command_handler::context::CommandContext;
use crate::{prefix_parser::Arguments, state::State};

pub type GlobalStateInner = State;
pub type GlobalState = Arc<GlobalStateInner>;

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
        let prefix_ctx = super::context::PrefixContext {
            message_id: message_ref.id,
            channel_id: message_ref.channel_id,
            parsed: arguments.clone(),
            prefix: prefix_str.clone(),
            http_client: state.http_client(),
            message: message_ref,
        };
        let cmd_ctx = CommandContext::Prefix(Box::new(prefix_ctx));

        if let Err(e) = Self::execute(state.clone(), cmd_ctx).await {
            let error_prefix_ctx = super::context::PrefixContext {
                message_id: message_ref.id,
                channel_id: message_ref.channel_id,
                parsed: arguments.clone(),
                prefix: prefix_str,
                http_client: state.http_client(),
                message: message_ref,
            };
            let error_cmd_ctx = CommandContext::Prefix(Box::new(error_prefix_ctx));
            if let Err(reply_err) = error_cmd_ctx
                .reply_error(&e, super::response::create_error_response)
                .await
            {
                tracing::error!(error = ?reply_err, "Failed to send error embed for prefix command");
            }
        }
        Ok(())
    }

    async fn execute_slash_command(
        state: S,
        interaction: Interaction,
        data: CommandData,
    ) -> anyhow::Result<()> {
        let interaction_for_error_reply = interaction.clone();
        let data_for_error_reply = data.clone();

        let slash_ctx = super::context::SlashContext {
            interaction,
            data,
            http_client: state.http_client(),
        };
        let cmd_ctx = CommandContext::Slash(Box::new(slash_ctx));

        if let Err(e) = Self::execute(state.clone(), cmd_ctx).await {
            let error_slash_ctx = super::context::SlashContext {
                interaction: interaction_for_error_reply,
                data: data_for_error_reply,
                http_client: state.http_client(),
            };
            let error_cmd_ctx = CommandContext::Slash(Box::new(error_slash_ctx));
            if let Err(reply_err) = error_cmd_ctx
                .reply_error(&e, super::response::create_error_response)
                .await
            {
                tracing::error!(error = ?reply_err, "Failed to send error embed for slash command");
            }
        }
        Ok(())
    }
}

pub struct CommandDefinition<S>
where
    S: HasHttpClient + StateExt + Clone + Send + Sync + 'static + Sized,
{
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub create_slash_data_fn: fn() -> twilight_model::application::command::Command,
    pub slash_executor:
        fn(S, Interaction, CommandData) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
    pub prefix_executor: for<'msg_lifetime> fn(
        S,
        &'msg_lifetime Message,
        Arguments<'msg_lifetime>,
        String,
    ) -> Pin<
        Box<dyn Future<Output = anyhow::Result<()>> + Send + 'msg_lifetime>,
    >,
}

#[macro_export]
macro_rules! command_def {
    ($state_type:ty, $command_type:ty) => {
        $crate::command_handler::CommandDefinition::<$state_type> {
            name: <$command_type as twilight_interactions::command::CreateCommand>::NAME,
            aliases: &[],
            create_slash_data_fn: || <$command_type as twilight_interactions::command::CreateCommand>::create_command().into(),
            slash_executor: |state, interaction, data| {
                Box::pin(<$command_type as $crate::command_handler::Command<$state_type>>::execute_slash_command(state, interaction, data))
            },
            prefix_executor: |state, message, args, prefix_str| {
                Box::pin(<$command_type as $crate::command_handler::Command<$state_type>>::execute_prefix_command(state, message, args, prefix_str))
            },
        }
    };
    ($state_type:ty, $command_type:ty, aliases = [$($alias:expr),* $(,)?]) => {
        $crate::command_handler::CommandDefinition::<$state_type> {
            name: <$command_type as twilight_interactions::command::CreateCommand>::NAME,
            aliases: &[$($alias),*],
            create_slash_data_fn: || <$command_type as twilight_interactions::command::CreateCommand>::create_command().into(),
            slash_executor: |state, interaction, data| {
                Box::pin(<$command_type as $crate::command_handler::Command<$state_type>>::execute_slash_command(state, interaction, data))
            },
            prefix_executor: |state, message, args, prefix_str| {
                Box::pin(<$command_type as $crate::command_handler::Command<$state_type>>::execute_prefix_command(state, message, args, prefix_str))
            },
        }
    };
}

pub trait HasHttpClient {
    fn http_client(&self) -> Arc<HttpClient>;
}

pub trait StateExt: HasHttpClient {
    fn lavalink(&self) -> Arc<LavalinkClient>;
    fn songbird(&self) -> Arc<Songbird>;
}
