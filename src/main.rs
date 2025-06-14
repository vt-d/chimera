mod command;
mod lavalink_events;
mod macros;
mod prefix_parser;
mod state;

use std::collections::HashMap;
use std::mem;
use std::{env, sync::Arc};

use dotenvy::dotenv;
use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::events;
use lavalink_rs::node::NodeBuilder;
use lavalink_rs::prelude::NodeDistributionStrategy;
use songbird::Songbird;
use songbird::shards::TwilightMap;
use twilight_gateway::{CloseFrame, Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::InteractionData;

use crate::state::State;

pub struct Bot {
    pub shard: Shard,
    pub state: Arc<State>,
}

impl Bot {
    pub fn new(shard: Shard, state: State) -> Self {
        Self {
            shard,
            state: Arc::new(state),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    dotenv()?;

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    // growing up is realizing that you shouldnt shard your tiny bot
    let initial_shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::GUILD_VOICE_STATES
            | Intents::MESSAGE_CONTENT,
    );

    let client = HttpClient::new(token);

    let user = client.current_user().await?.model().await?;
    let user_id = user.id;
    let lavalink_events = events::Events {
        ready: Some(lavalink_events::ready_event),
        ..Default::default()
    };

    let node_local = NodeBuilder {
        hostname: "0.0.0.0:2333".to_string(),
        is_ssl: false,
        events: events::Events::default(),
        password: std::env::var("LAVALINK_PASSWORD").unwrap_or_default(),
        user_id: user_id.into(),
        session_id: None,
    };

    let lavalink = LavalinkClient::new(
        lavalink_events,
        vec![node_local],
        NodeDistributionStrategy::round_robin(),
    )
    .await;

    let senders = TwilightMap::new(HashMap::from([(
        initial_shard.id().number(),
        initial_shard.sender(),
    )]));

    let songbird = Songbird::twilight(Arc::new(senders), user_id);

    let state = State {
        http: Arc::new(client),
        lavalink: Arc::new(lavalink),
        songbird: Arc::new(songbird),
    };
    let mut bot = Bot::new(initial_shard, state);

    register_commands(&bot).await?;

    tracing::info!("Bot is running. Press Ctrl+C to exit.");

    tokio::select! {
        runner_result = runner(&mut bot) => {
            match runner_result {
                Ok(_) => {
                    tracing::info!("Runner finished successfully.");
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Runner failed");
                    return Err(e);
                }
            }
        },
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received, shutting down...");
        }
    };

    tracing::info!("Closing shard connection...");
    bot.shard.close(CloseFrame::NORMAL);

    Ok(())
}

async fn runner(bot: &mut Bot) -> anyhow::Result<()> {
    let shard = &mut bot.shard;
    let state = &bot.state;
    let configured_prefix = env::var("PREFIX").unwrap_or_else(|_| ";".to_string());

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(Event::GatewayClose(_)) => {
                tracing::info!("Gateway connection closed, exiting runner.");
                break;
            }
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "error receiving event");
                tracing::error!("Error receiving event from shard, breaking loop for now.");
                break;
            }
        };

        match event {
            Event::InteractionCreate(interaction_payload) => {
                let mut interaction_obj = interaction_payload.0;

                let data = match mem::take(&mut interaction_obj.data) {
                    Some(InteractionData::ApplicationCommand(data)) => *data,
                    _ => {
                        tracing::warn!("ignoring non-application-command interaction");
                        continue;
                    }
                };

                if let Err(error) = crate::command::slash_handler(
                    interaction_obj,
                    data,
                    bot.state.http.clone(),
                    bot.state.clone(),
                )
                .await
                {
                    tracing::error!(?error, "error while handling slash command");
                }
            }
            Event::MessageCreate(message_payload) => {
                let message = message_payload.0;

                if message.author.bot {
                    continue;
                }
                if let Err(error) = crate::command::prefix_handler(
                    message,
                    bot.state.http.clone(),
                    &configured_prefix,
                    bot.state.clone(),
                )
                .await
                {
                    tracing::error!(?error, "error while handling prefix command");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn register_commands(bot: &Bot) -> anyhow::Result<()> {
    let commands = vec![crate::command::ping::PingCommand::create_command().into()];
    let application = bot
        .state
        .http
        .current_user_application()
        .await?
        .model()
        .await?;
    let interaction_client = bot.state.http.interaction(application.id);

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }
    Ok(())
}
