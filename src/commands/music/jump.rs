use super::check_voice_state;
use crate::command_handler::{
    Command, CommandContext, CommandResponseBuilder, GlobalState, StateExt,
};
use crate::prefix_parser::Arguments;
use anyhow::Result;
use async_trait::async_trait;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::application_command::CommandOptionValue;

#[allow(unused)]
#[derive(CommandModel, CreateCommand)]
#[command(name = "jump", desc = "Jump to a specific track in the queue.")]
pub struct JumpCommand {
    #[command(
        desc = "The queue position to jump to (0 for the first song, 1 for the second, etc.)."
    )]
    position: i64,
}

#[async_trait]
impl Command<GlobalState> for JumpCommand {
    async fn execute<'ctx>(state: GlobalState, cmd_ctx: CommandContext<'ctx>) -> Result<()> {
        let guild_id = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx) => prefix_ctx.message.guild_id,
            CommandContext::Slash(slash_ctx) => slash_ctx.interaction.guild_id,
        }
        .ok_or_else(|| anyhow::anyhow!("This command must be used in a guild."))?;

        let position_i64 = match &cmd_ctx {
            CommandContext::Prefix(prefix_ctx_ref) => {
                let mut local_args = Arguments::new(prefix_ctx_ref.parsed.remainder());
                local_args.next().and_then(|arg| arg.parse::<i64>().ok())
            }
            CommandContext::Slash(slash_ctx_ref) => slash_ctx_ref
                .data
                .options
                .iter()
                .find(|opt| opt.name == "position")
                .and_then(|opt| match &opt.value {
                    CommandOptionValue::Integer(i_val) => Some(*i_val),
                    _ => None,
                }),
        }
        .ok_or_else(|| anyhow::anyhow!("Position argument is missing or invalid. Please provide a number (e.g., 0 for the first song)."))?;

        if position_i64 < 0 {
            return Err(anyhow::anyhow!(
                "Position to jump to cannot be negative. Use 0 for the first song."
            ));
        }
        let target_idx = position_i64 as usize;

        check_voice_state(state.clone(), &cmd_ctx).await?;

        let player_context = state
            .lavalink()
            .get_player_context(guild_id)
            .ok_or_else(|| anyhow::anyhow!("I'm not playing anything in this guild."))?;

        let queue = player_context.get_queue();
        let initial_queue_count = queue.get_count().await?;

        if initial_queue_count == 0 {
            return Err(anyhow::anyhow!(
                "The queue is currently empty. Cannot jump."
            ));
        }

        if target_idx >= initial_queue_count {
            return Err(anyhow::anyhow!(
                "Cannot jump to position {}. The queue only has {} tracks (indexed 0 to {}).",
                target_idx,
                initial_queue_count,
                initial_queue_count.saturating_sub(1)
            ));
        }

        for i in 0..target_idx {
            match queue.remove(0) {
                Ok(_) => {
                    tracing::debug!(guild_id = %guild_id, "Removed track at index 0 (iteration {} of {} for jump)", i, target_idx);
                }
                Err(e) => {
                    tracing::error!(error = ?e, guild_id = %guild_id, "Failed to remove track from queue during jump");
                    return Err(anyhow::anyhow!(
                        "An unexpected error occurred while modifying the queue."
                    ));
                }
            }
        }

        if queue.get_count().await? > 0 {
            if let Err(e) = player_context.skip() {
                tracing::warn!(error = ?e, guild_id = %guild_id, "Failed to explicitly skip to jumped track, but queue was modified.");
            }
        } else {
            tracing::info!(guild_id = %guild_id, "Queue became empty after jump operation for position {}. Player might stop or be cleared.", target_idx);
        }

        let response = CommandResponseBuilder::new()
            .content(format!(
                "⬆️ Jumped to track at position {} in the queue.",
                position_i64
            ))
            .build();

        cmd_ctx.reply(response).await?;

        Ok(())
    }
}
