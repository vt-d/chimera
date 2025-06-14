use std::sync::Arc;

use songbird::Songbird;
use twilight_http::Client;

use crate::command::{HasHttpClient, StateExt};

#[derive(Clone)]
pub struct State {
    pub http: Arc<twilight_http::Client>,
    pub lavalink: Arc<lavalink_rs::client::LavalinkClient>,
    pub songbird: Arc<Songbird>,
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
