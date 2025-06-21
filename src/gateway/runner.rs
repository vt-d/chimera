use tokio::sync::oneshot;
use twilight_gateway::{CloseFrame, Event, EventTypeFlags, StreamExt};

use super::{Bot, ShardInfo};

#[tracing::instrument(skip(bot, shutdown_rx))]
pub async fn runner(mut bot: Bot, mut shutdown_rx: oneshot::Receiver<()>) -> anyhow::Result<()> {
    let shard_info_sender = bot.shard_info_tx.clone();

    loop {
        tokio::select! {
            biased;

            _ = &mut shutdown_rx => {
                tracing::info!("Gateway runner received shutdown signal. Exiting event loop.");
                break;
            }

            item = bot.shard.next_event(EventTypeFlags::all()) => {
                let event = match item {
                    None => {
                        tracing::info!("Shard event stream ended. Runner will exit.");
                        break;
                    }
                    Some(Ok(event)) => event,
                    Some(Err(source)) => {
                        tracing::warn!(?source, "Error receiving event from shard");
                        continue;
                    }
                };

                if let Event::GatewayClose(frame) = &event {
                    match shutdown_rx.try_recv() {
                        Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                            tracing::info!(?frame, "Gateway connection closed during planned shutdown.");
                        }
                        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                            tracing::warn!(?frame, "Gateway connection closed unexpectedly by Discord. The runner will exit as this is non-resumable.");
                        }
                    }
                    // GatewayClose is always a terminal event for the shard's event stream, so we must exit the loop.
                    break;
                }

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
                    let _ = state_clone.cache.update(&event);

                    if let Err(e) = super::process(event, state_clone).await {
                        tracing::error!(error = ?e, "Error processing event");
                    }
                });
            }
        }
    }

    tracing::info!("Gateway runner loop ended. Closing shard...");
    bot.shard.close(CloseFrame::NORMAL);

    Ok(())
}
