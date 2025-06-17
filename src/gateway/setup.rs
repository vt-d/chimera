use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use dashmap::DashMap;
use dotenvy::dotenv;
use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::events as LavalinkEventsModel;
use lavalink_rs::node::NodeBuilder;
use lavalink_rs::prelude::NodeDistributionStrategy;
use songbird::Songbird;
use songbird::shards::TwilightMap;
use tokio::sync::{mpsc, oneshot};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use twilight_gateway::{ConfigBuilder, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{ActivityType, MinimalActivity, Status};
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::voice::VoiceState;

use crate::commands::COMMANDS;
use crate::config::Config;
use crate::gateway::runner;
use crate::lavalink_events;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct ShardInfo {
    pub latency_ms: Option<u128>,
}

pub struct Bot {
    pub shard: Shard,
    pub state: Arc<State>,
    pub voice_states: Arc<DashMap<Id<UserMarker>, VoiceState>>,
    pub shard_info_tx: mpsc::Sender<ShardInfo>,
}

impl Bot {
    pub fn new(
        shard: Shard,
        state: Arc<State>,
        voice_states: Arc<DashMap<Id<UserMarker>, VoiceState>>,
        shard_info_tx: mpsc::Sender<ShardInfo>,
    ) -> Self {
        Self {
            shard,
            state,
            voice_states,
            shard_info_tx,
        }
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set global default tracing subscriber: {}", e))?;
    Ok(())
}

fn load_config_and_env_sync() -> anyhow::Result<Config> {
    dotenv().map_err(|e| anyhow::anyhow!("Failed to load .env file: {}", e))?;
    Config::from_env()
}

fn init_http_client(config: &Config) -> Arc<HttpClient> {
    Arc::new(HttpClient::new(config.token.clone()))
}

fn init_shard(config: &Config, presence: UpdatePresencePayload) -> Shard {
    let config = ConfigBuilder::new(
        config.token.clone(),
        Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::GUILD_VOICE_STATES
            | Intents::MESSAGE_CONTENT,
    )
    .presence(presence)
    .build();
    Shard::with_config(ShardId::ONE, config)
}

fn presence() -> anyhow::Result<UpdatePresencePayload> {
    Ok(UpdatePresencePayload::new(
        [MinimalActivity {
            name: "music".to_string(),
            kind: ActivityType::Listening,
            url: None,
        }
        .into()],
        false,
        None,
        Status::Idle,
    )?)
}

async fn init_lavalink_client(
    config: &Config,
    user_id: Id<UserMarker>,
) -> anyhow::Result<Arc<LavalinkClient>> {
    let lavalink_events_handlers = LavalinkEventsModel::Events {
        ready: Some(lavalink_events::ready_event),
        raw: Some(lavalink_events::raw_event),
        ..Default::default()
    };

    let node_local = NodeBuilder {
        hostname: config.lavalink_host.clone(),
        is_ssl: false,
        events: LavalinkEventsModel::Events::default(),
        password: config.lavalink_password.clone(),
        user_id: user_id.into(),
        session_id: None,
    };

    let client = LavalinkClient::new(
        lavalink_events_handlers,
        vec![node_local],
        NodeDistributionStrategy::round_robin(),
    )
    .await;
    Ok(Arc::new(client))
}

async fn init_songbird_client(
    shard_sender: twilight_gateway::MessageSender,
    shard_id_number: u32,
    user_id: Id<UserMarker>,
) -> anyhow::Result<Arc<Songbird>> {
    let senders = TwilightMap::new(HashMap::from([(shard_id_number, shard_sender)]));
    Ok(Arc::new(Songbird::twilight(Arc::new(senders), user_id)))
}

fn init_app_state(
    http: Arc<HttpClient>,
    lavalink: Arc<LavalinkClient>,
    songbird: Arc<Songbird>,
    config: Config,
) -> Arc<State> {
    Arc::new(crate::state::State::new(http, lavalink, songbird, config))
}

async fn register_bot_commands(state: Arc<State>) -> anyhow::Result<()> {
    let commands_to_register: Vec<twilight_model::application::command::Command> = COMMANDS
        .iter()
        .map(|cmd_def| (cmd_def.create_slash_data_fn)())
        .collect();

    if commands_to_register.is_empty() {
        tracing::info!("No commands to register.");
        return Ok(());
    }

    let application_id = state
        .http
        .current_user_application()
        .await
        .context("Failed to get current user application")?
        .model()
        .await
        .context("Failed to model current user application")?
        .id;

    let interaction_client = state.http.interaction(application_id);

    match interaction_client
        .set_global_commands(&commands_to_register)
        .await
    {
        Ok(_) => {
            tracing::info!("Successfully registered global commands.");
        }
        Err(error) => {
            tracing::error!(?error, "Failed to register global commands");
        }
    }
    Ok(())
}

pub async fn initialize_and_run_bot() -> anyhow::Result<()> {
    init_tracing().context("Failed to initialize tracing")?;
    tracing::info!("Chimera Bot starting up (from gateway::setup)...");

    let (shard_info_tx, mut shard_info_rx) = mpsc::channel::<ShardInfo>(32);

    let config =
        load_config_and_env_sync().context("Failed to load configuration and .env file")?;

    let http_client = init_http_client(&config);

    let current_user_id = http_client
        .current_user()
        .await
        .context("Failed to get current user from Discord")?
        .model()
        .await
        .context("Failed to model current user data")?
        .id;

    let lavalink_client = init_lavalink_client(&config, current_user_id)
        .await
        .context("Failed to initialize Lavalink client")?;

    let initial_shard = init_shard(&config, presence()?);

    let songbird_client = init_songbird_client(
        initial_shard.sender(),
        initial_shard.id().number() as u32,
        current_user_id,
    )
    .await
    .context("Failed to initialize Songbird client")?;

    let voice_states_map = Arc::new(DashMap::new());

    let app_state = init_app_state(
        http_client.clone(),
        lavalink_client,
        songbird_client,
        config.clone(),
    );

    let bot = Bot::new(
        initial_shard,
        app_state.clone(),
        voice_states_map.clone(),
        shard_info_tx,
    );

    register_bot_commands(app_state.clone())
        .await
        .context("Failed to register bot commands")?;

    let app_state_for_latency_task = app_state.clone();
    tokio::spawn(async move {
        while let Some(info) = shard_info_rx.recv().await {
            if let Some(latency_value) = info.latency_ms {
                let mut latency_state = app_state_for_latency_task.latency_ms.lock().await;
                *latency_state = Some(latency_value);
                tracing::debug!("Latency updated: {}ms", latency_value);
            }
        }
        tracing::info!("Shard info receiver channel closed, latency updates will stop.");
    });

    tracing::info!("Bot initialized. Connecting to gateway and running event loop...");

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let runner_handle = tokio::spawn(async move { runner(bot, shutdown_rx).await });

    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!(error = ?e, "Failed to listen for ctrl_c signal");
        let _ = shutdown_tx.send(());
    } else {
        tracing::info!("Ctrl+C received. Initiating graceful shutdown...");
        if shutdown_tx.send(()).is_err() {
            tracing::warn!(
                "Failed to send shutdown signal to gateway runner; it might have already exited."
            );
        }
    }

    tracing::info!("Waiting for gateway runner to complete...");
    match runner_handle.await {
        Ok(Ok(_)) => tracing::info!("Gateway runner finished successfully."),
        Ok(Err(e)) => tracing::error!(error = ?e, "Gateway runner failed."),
        Err(e) => tracing::error!(error = ?e, "Gateway runner task panicked or was cancelled."),
    }

    tracing::info!("Shutdown complete.");
    Ok(())
}
