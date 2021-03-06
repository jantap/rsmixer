extern crate crossbeam_channel as cb_channel;
extern crate libpulse_binding as pulse;

mod config;
mod errors;
mod event_loop;
mod help;
mod input;
mod models;
mod pa;
mod run;
mod ui;

pub use errors::RsError;
pub use models::{entry, Action};

use config::{RsMixerConfig, Variables};
use ev_apple::{Dispatch, Senders};
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
    pub static ref DISPATCH: Dispatch<Action> = Dispatch::default();
    pub static ref SENDERS: Senders<Action> = Senders::default();
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

    run::run().await
}

fn main() -> Result<(), RsError> {
    let mut threaded_rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    threaded_rt.block_on(async {
        if let Err(err) = launch().await {
            eprintln!("{:#}", err);
        }
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
