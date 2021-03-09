extern crate crossbeam_channel as cb_channel;
extern crate libpulse_binding as pulse;

mod action_handlers;
mod actor_system;
mod config;
mod errors;
mod event_loop_actor;
mod help;
mod input_actor;
mod models;
mod multimap;
mod pa;
mod pa_actor;
mod ui;

pub use errors::RsError;
pub use models::{entry, Action};

use config::{RsMixerConfig, Variables};
use models::{InputEvent, Style};

use tokio::runtime;

use crossterm::style::ContentStyle;

use log::LevelFilter;

use lazy_static::lazy_static;

use state::Storage;

use gumdrop::Options;

use multimap::MultiMap;
use std::collections::HashMap;

lazy_static! {
    pub static ref STYLES: Storage<Styles> = Storage::new();
    pub static ref VARIABLES: Storage<Variables> = Storage::new();
    pub static ref BINDINGS: Storage<MultiMap<InputEvent, Action>> = Storage::new();
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Styles = HashMap<Style, ContentStyle>;

#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "filepath to log to (if left empty the program doesn't log)")]
    log_file: Option<String>,

    #[options(count, help = "verbosity. Once - info, twice - debug")]
    verbose: usize,

    #[options(help = "show this text")]
    help: bool,
}

async fn launch() -> Result<(), RsError> {
    let opts = CliOptions::parse_args_default_or_exit();

    if opts.help {
        println!("{}", CliOptions::usage());
        return Ok(());
    }

    if let Some(file) = opts.log_file {
        let lvl = match opts.verbose {
            2 => LevelFilter::Debug,
            1 => LevelFilter::Info,
            _ => LevelFilter::Warn,
        };
        simple_logging::log_to_file(file, lvl).unwrap();
    }

    let mut config = RsMixerConfig::load()?;

    let (styles, bindings, variables) = config.interpret()?;

    STYLES.set(styles);
    BINDINGS.set(bindings);
    VARIABLES.set(variables);

    Ok(())
}

fn main() -> Result<(), RsError> {
    let (mut context, worker) = actor_system::new();

    let mut threaded_rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    threaded_rt.block_on(async {
        launch().await.unwrap();
        let x = worker.start();
        context.actor("event_loop", &event_loop_actor::EventLoop::new);
        context.actor("pulseaudio", &pa_actor::PulseActor::new);
        context.actor("input", &input_actor::InputActor::new);

        x.await.unwrap().unwrap();
    });

    Ok(())
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($x:expr, $y:expr) => {
        match $x {
            Some(x) => x,
            None => {
                return $y;
            }
        }
    };
    ($x:expr) => {
        unwrap_or_return!($x, ())
    };
}
