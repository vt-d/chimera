mod process;
mod runner;
pub mod setup;

use setup::*;

pub use process::process;
pub use runner::runner;
pub use setup::initialize_and_run_bot;
