use super::check_voice_state;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt,
};
use anyhow::Result;
use async_trait::async_trait;
use lavalink_rs::player_context::QueueRef;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::EmbedBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(name = "queue", desc = "Show the current music queue.")]
pub struct QueueCommand;

#[async_trait]
impl Command<GlobalState> for QueueCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?;

        check_voice_state(state.clone(), &cmd_ctx).await?;

        let player = state
            .lavalink()
            .get_player_context(guild_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for this guild."))?;
        let queue = player.get_queue();

        let embed = build_queue_embed(&queue).await?;
        let response = CommandResponseBuilder::new().embed(embed).build();

        cmd_ctx.reply(response).await?;

        Ok(())
    }
}

pub async fn build_queue_embed(queue: &QueueRef) -> Result<Embed> {
    let mut embed = EmbedBuilder::new()
        .title("🎶 Current Queue")
        .color(0x1DB954);

    if queue.get_count().await? == 0 {
        embed = embed.description("The queue is currently empty.");
    } else {
        let queue_list: Vec<String> = queue
            .get_queue()
            .await?
            .iter()
            .map(|track| format!("{} - {}", track.track.info.title, track.track.info.author))
            .collect();
        embed = embed.description(format!("{}\n", queue_list.join("\n")));
    }

    Ok(embed.build())
}
