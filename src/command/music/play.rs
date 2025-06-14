use crate::{
    command::{Command, CommandContext, GlobalState}
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::application_command::CommandOptionValue;

#[derive(CreateCommand)]
#[command(name = "play", desc = "Play a song from YouTube or other sources.")]
pub struct PlayCommand {
    #[command(desc = "The song to play")]
    pub song: String,
}

#[async_trait]
impl Command<GlobalState> for PlayCommand {
    async fn execute<'ctx>(_state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let song_query = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => Ok(prefix_ctx.parsed.remainder().to_string()),
            CommandContext::Slash(slash_ctx) => slash_ctx
                .data
                .options
                .get(0)
                .ok_or_else(|| anyhow!("The 'song' option is missing for the slash command."))
                .and_then(|option| match &option.value {
                    CommandOptionValue::String(s_val) => Ok(s_val.clone()),
                    _ => Err(anyhow!("The 'song' option must be a string.")),
                }),
        }?;

        cmd_ctx
            .reply(format!("Now playing: {}", song_query))
            .await?;
        Ok(())
    }
}
