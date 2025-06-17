use std::sync::Arc;

use dashmap::DashMap;
use songbird::Songbird;
use twilight_http::Client;
use twilight_model::voice::VoiceState;

use crate::command_handler::{HasHttpClient, StateExt};
use crate::config::Config;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct State {
    pub http: Arc<twilight_http::Client>,
    pub lavalink: Arc<lavalink_rs::client::LavalinkClient>,
    pub songbird: Arc<Songbird>,
    pub voice_states:
        Arc<DashMap<twilight_model::id::Id<twilight_model::id::marker::UserMarker>, VoiceState>>,
    pub config: Config,
    pub latency_ms: Arc<Mutex<Option<u128>>>,
    pub reqwest: Arc<reqwest::Client>,
}

impl HasHttpClient for State {
    fn http_client(&self) -> Arc<Client> {
        self.http.clone()
    }
}

impl StateExt for State {
    fn lavalink(&self) -> Arc<lavalink_rs::client::LavalinkClient> {
        self.lavalink.clone()
    }

    fn songbird(&self) -> Arc<Songbird> {
        self.songbird.clone()
    }
}

impl HasHttpClient for Arc<State> {
    fn http_client(&self) -> Arc<Client> {
        self.as_ref().http_client()
    }
}

impl StateExt for Arc<State> {
    fn lavalink(&self) -> Arc<lavalink_rs::client::LavalinkClient> {
        self.as_ref().lavalink()
    }

    fn songbird(&self) -> Arc<Songbird> {
        self.as_ref().songbird()
    }
}

impl State {
    pub fn new(
        http: Arc<twilight_http::Client>,
        lavalink: Arc<lavalink_rs::client::LavalinkClient>,
        songbird: Arc<Songbird>,
        config: Config,
    ) -> Self {
        Self {
            http,
            lavalink,
            songbird,
            voice_states: Arc::new(DashMap::new()),
            config,
            latency_ms: Arc::new(Mutex::new(None)),
            reqwest: Arc::new(reqwest::Client::new()),
        }
    }
}
