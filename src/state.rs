use std::sync::Arc;

use songbird::Songbird;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_http::Client;

use crate::command_handler::{HasHttpClient, StateExt};
use crate::config::Config;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct State {
    pub http: Arc<twilight_http::Client>,
    pub lavalink: Arc<lavalink_rs::client::LavalinkClient>,
    pub songbird: Arc<Songbird>,
    pub cache: Arc<InMemoryCache>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        http: Arc<twilight_http::Client>,
        lavalink: Arc<lavalink_rs::client::LavalinkClient>,
        songbird: Arc<Songbird>,
        config: Config,
        reqwest: Arc<reqwest::Client>,
    ) -> Self {
        const CACHE_EVENTS: ResourceType = ResourceType::GUILD
            .union(ResourceType::VOICE_STATE);

        Self {
            http,
            lavalink,
            songbird,
            cache: Arc::new(InMemoryCache::builder().resource_types(CACHE_EVENTS).build()),
            config,
            latency_ms: Arc::new(Mutex::new(None)),
            reqwest,
        }
    }
}
