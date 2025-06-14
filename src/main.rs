mod command;
mod prefix_parser;

use std::mem;
use std::{env, sync::Arc};

use dotenvy::dotenv;
use twilight_gateway::{Event, EventTypeFlags, Shard, ShardId, StreamExt, Intents, CloseFrame};
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::InteractionData;

use crate::command::{HasHttpClient, StateExt};


pub struct Bot {
    pub shard: Shard,
    pub client: Arc<HttpClient>,
}

impl Bot {
    pub fn new(shard: Shard, client: HttpClient) -> Self {
        Self { shard, client: Arc::new(client) }
    }
}

#[derive(Clone)]
pub struct State {
    pub http: Arc<twilight_http::Client>,
}

impl HasHttpClient for State {
    fn http_client(&self) -> Arc<HttpClient> {
        self.http.clone()
    }
}

impl StateExt for State {}

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
        Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES | Intents::MESSAGE_CONTENT,
    );

    let client = HttpClient::new(token);
    let mut bot = Bot::new(initial_shard, client);

    register_commands(&bot).await?;

    tracing::info!("Bot is running. Press Ctrl+C to exit.");

    let bot = tokio::select! {
        runner_result = runner(&mut bot) => {
            match runner_result {
                Ok(_) => {
                    tracing::info!("Runner finished successfully.");
                    bot
                }
                Err(e) => {
                    tracing::error!("Runner failed: {:?}", e);
                    return Err(e);
                }
            }
        },
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received, shutting down...");
            bot
        }
    };

    bot.shard.close(CloseFrame::NORMAL);

    Ok(())
}

async fn runner(bot: &mut Bot) -> anyhow::Result<()> {
    let shard = &mut bot.shard;
    let configured_prefix = env::var("PREFIX").unwrap_or_else(|_| "!".to_string());

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

                if let Err(error) = crate::command::slash_handler(interaction_obj, data, bot.client.clone()).await {
                    tracing::error!(?error, "error while handling slash command");
                }
            }
            Event::MessageCreate(message_payload) => {
                let message = message_payload.0; 

                if message.author.bot {
                    continue; 
                }
                if let Err(error) = crate::command::prefix_handler(message, bot.client.clone(), &configured_prefix).await {
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
    let application = bot.client.current_user_application().await?.model().await?;
    let interaction_client = bot.client.interaction(application.id);

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }
    Ok(())
}

