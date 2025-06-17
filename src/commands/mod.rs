pub mod music;
pub mod ping;

use crate::{
    command_def,
    command_handler::{CommandDefinition, GlobalState},
};
use once_cell::sync::Lazy;

use music::*;
use ping::PingCommand;

pub static COMMANDS: Lazy<Vec<CommandDefinition<GlobalState>>> = Lazy::new(|| {
    vec![
        command_def!(GlobalState, PingCommand),
        command_def!(GlobalState, PlayCommand, 
            aliases = ["p", "play"]),
        command_def!(GlobalState, StopCommand, 
            aliases = ["st", "stop"]),
        command_def!(GlobalState, QueueCommand, 
            aliases = ["q", "queue"]),
        command_def!(GlobalState, NowPlayingCommand, 
            aliases = ["np", "nowplaying"]),
        command_def!(GlobalState, SkipCommand, 
            aliases = ["s", "skip"]),
    ]
});
