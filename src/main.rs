extern crate crossbeam_channel as cb_channel;
extern crate libpulse_binding as pulse;

mod config;
mod entry;
mod errors;
mod events;
mod helpers;
mod input;
mod pa;
mod ui;

pub use errors::RSError;
pub use events::Letter;

use config::RsMixerConfig;
use events::{Dispatch, Message, Senders};
use pa::INFO_SX;

use std::{collections::HashMap, io::Write};

use tokio::runtime;
use tokio::sync::{broadcast::channel, mpsc};
use tokio::task;

use crossterm::{event::KeyEvent, style::ContentStyle};

use log::LevelFilter;

use lazy_static::lazy_static;

use state::Storage;

use gumdrop::Options;

lazy_static! {
    pub static ref DISPATCH: Dispatch<Letter> = Dispatch::default();
    pub static ref SENDERS: Senders<Letter> = Senders::default();
    pub static ref STYLES: Storage<Styles> = Storage::new();
    pub static ref BINDINGS: Storage<HashMap<KeyEvent, Letter>> = Storage::new();
}

pub type Styles = HashMap<String, ContentStyle>;

#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "filepath to log to (if left empty the program doesn't log)")]
    log_file: Option<String>,

    #[options(count, help = "verbosity. Once - info, twice - debug")]
    verbose: usize,

    #[options(help = "show this text")]
    help: bool,
}

async fn run() -> Result<(), RSError> {
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

    let config: RsMixerConfig = confy::load("rsmixer")?;

    let (styles, bindings) = config.load()?;

    STYLES.set(styles);
    BINDINGS.set(bindings);

    let hl = helpers::help_text::generate();

    let (event_sx, event_rx) = channel(32);
    let (r, s) = cb_channel::unbounded();
    let event2 = event_sx.clone();
    DISPATCH.register(event_sx, r.clone()).await;

    task::spawn(async move { events::start(event2, event_rx, s, r, SENDERS.clone()).await });

    let ui = async move {
        match task::spawn(async move { ui::start().await }).await {
            Ok(r) => r,
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let (pa_sx, pa_rx) = cb_channel::unbounded();

    let (info_sx, info_rx) = mpsc::unbounded_channel();
    (*INFO_SX).set(info_sx);

    let pa = async move {
        match task::spawn_blocking(move || pa::start(pa_rx)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let pa_async = async move {
        match task::spawn(async move { pa::start_async(pa_sx, info_rx).await }).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let r = tokio::try_join!(ui, pa, pa_async);

    DISPATCH.event(Letter::ExitSignal).await;

    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    )
    .unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();

    match r {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn main() -> Result<(), RSError> {
    let mut threaded_rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    threaded_rt.block_on(async {
        if let Err(err) = run().await {
            eprintln!("{}", err);
        }
    });

    Ok(())
}
