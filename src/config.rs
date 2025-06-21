use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub configured_prefix: String,
    pub token: String,
    pub lavalink_host: String,
    pub lavalink_port: u16,
    pub lavalink_password: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let configured_prefix = env::var("PREFIX").unwrap_or_else(|_| ";".to_string());
        let token = env::var("DISCORD_TOKEN")?;
        let lavalink_host = env::var("LAVALINK_HOST")?;
        let lavalink_port = env::var("LAVALINK_PORT")?.parse()?;
        let lavalink_password = env::var("LAVALINK_PASSWORD")?;
        Ok(Self {
            configured_prefix,
            token,
            lavalink_host,
            lavalink_port,
            lavalink_password,
        })
    }
}
