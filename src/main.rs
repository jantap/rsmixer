extern crate crossbeam_channel as cb_channel;
extern crate libpulse_binding as pulse;

static LOGGING_MODULE: &'static str = "Main";

mod action_handlers;
mod actor_system;
mod actors;
mod cli_options;
mod config;
mod errors;
mod help;
mod models;
mod multimap;
mod pa;
mod prelude;
mod ui;
mod util;

pub use errors::RsError;
pub use models::{entry, Action};

use prelude::*;
use cli_options::CliOptions;
use config::{RsMixerConfig, Variables};
use actors::*;
use models::{InputEvent, Style};

use tokio::runtime;

use crossterm::style::ContentStyle;

use lazy_static::lazy_static;

use state::Storage;

use multimap::MultiMap;
use std::collections::HashMap;

lazy_static! {
    pub static ref STYLES: Storage<Styles> = Storage::new();
    pub static ref VARIABLES: Storage<Variables> = Storage::new();
    pub static ref BINDINGS: Storage<MultiMap<InputEvent, Action>> = Storage::new();
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Styles = HashMap<Style, ContentStyle>;

fn load_config_and_options() -> Result<()> {
    info!("Checking command line options and config");

    CliOptions::check()?;
    debug!("CLI options checked");

    let mut config = RsMixerConfig::load()?;
    let (styles, bindings, variables) = config.interpret()?;

    STYLES.set(styles);
    BINDINGS.set(bindings);
    VARIABLES.set(variables);
    debug!("Config loaded");

    Ok(())
}

async fn run() -> Result<()> {
    load_config_and_options()?;

    debug!("Starting actor system");
    let (mut context, worker) = actor_system::new();

    let actor_system_handle = worker.start();

    context.actor("event_loop", &EventLoopActor::new);
    context.actor("pulseaudio", &PulseActor::new);
    context.actor("input", &InputActor::new);

    debug!("Actor system started");
    actor_system_handle.await?
}

fn main() -> Result<()> {
    info!("Starting RsMixer");

    let mut threaded_rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    threaded_rt.block_on(async {
        debug!("Tokio runtime started");
        
        match run().await {
            Err(e) => println!("{:#?}", e),
            _ => {}
        }
    });

    Ok(())
}
