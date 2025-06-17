use tokio::sync::oneshot;
use twilight_gateway::{CloseFrame, Event, EventTypeFlags, StreamExt};

use super::{Bot, ShardInfo};

#[tracing::instrument(skip(bot, shutdown_rx))]
pub async fn runner(mut bot: Bot, mut shutdown_rx: oneshot::Receiver<()>) -> anyhow::Result<()> {
    let shard_info_sender = bot.shard_info_tx.clone();

    loop {
        tokio::select! {

            _ = &mut shutdown_rx => {
                tracing::info!("Gateway runner received shutdown signal. Exiting event loop.");
                break;
            }

            item = bot.shard.next_event(EventTypeFlags::all()) => {
                let event = match item {
                    Some(Ok(Event::GatewayClose(frame))) => {
                        tracing::info!(?frame, "Gateway connection closed by Discord, exiting runner.");
                        return Ok(());
                    }
                    Some(Ok(event)) => event,
                    Some(Err(source)) => {
                        tracing::warn!(?source, "Error receiving event from shard");
                        break;
                    }
                    None => {
                        tracing::info!("Shard event stream ended.");
                        break;
                    }
                };

                if let Event::GatewayHeartbeatAck = &event {
                    let latency_obj = bot.shard.latency();
                    if let Some(duration) = latency_obj.average() {
                        let shard_info = ShardInfo {
                            latency_ms: Some(duration.as_millis()),
                        };
                        if let Err(e) = shard_info_sender.send(shard_info).await {
                            tracing::warn!("Failed to send shard info: {}", e);
                        }
                    } else {
                        tracing::debug!(
                            "No latency data found in average after GatewayHeartbeatAck for shard {}",
                            bot.shard.id().number()
                        );
                    }
                }

                let state_clone = bot.state.clone();

                tokio::spawn(async move {
                    let _ = state_clone.songbird.process(&event).await;

                    if let Err(e) = super::process(event, state_clone).await {
                        tracing::error!(error = ?e, "Error processing event");
                    }
                });
            }
        }
    }

    tracing::info!("Gateway runner loop ended. Closing shard...");
    bot.shard.close(CloseFrame::NORMAL);
    tracing::info!("Shard closed successfully.");

    Ok(())
}
