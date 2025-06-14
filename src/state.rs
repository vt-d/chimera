use std::sync::Arc;

use twilight_http::Client;

use crate::command::{HasHttpClient, StateExt};

#[derive(Clone)]
pub struct State {
    pub http: Arc<twilight_http::Client>,
}

impl HasHttpClient for State {
    fn http_client(&self) -> Arc<Client> {
        self.http.clone()
    }
}

impl StateExt for State {}