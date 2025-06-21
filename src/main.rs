#![deny(clippy::all, clippy::pedantic)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod command_handler;
pub mod commands;
pub mod components;
pub mod config;
pub mod gateway;
pub mod lavalink_events;
pub mod prefix_parser;
pub mod state;
pub mod utils;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    print_banner();
    crate::gateway::initialize_and_run_bot().await
}

const BANNER: &str = r#"
       .__    .__                             
  ____ |  |__ |__| _____   ________________   
_/ ___\|  |  \|  |/     \_/ __ \_  __ \__  \  
\  \___|   Y  \  |  Y Y  \  ___/|  | \// __ \_
 \___  >___|  /__|__|_|  /\___  >__|  (____  /
     \/     \/         \/     \/           \/ 

Chimera - Non-trash, blazingly fast music bot
"#;

fn print_banner() {
    let git_hash = env!("GIT_HASH");
    let build_time = env!("BUILD_TIME");
    let version = env!("APP_VERSION");

    println!("{}", BANNER);
    println!("  Version   : {}", version);
    println!("  Commit    : {}", git_hash);
    println!("  Built at  : {}", build_time);
    println!("  https://github.com/vt-d/chimera");
    println!("  (C) 2025 vt-d");
    println!("--------------------------------------------------------------\n");
}
