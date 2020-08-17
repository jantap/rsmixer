extern crate crossbeam_channel as cb_channel;
extern crate libpulse_binding as pulse;

mod config;
mod entry;
mod errors;
mod events;
mod input;
mod pa;
mod ui;

pub use errors::RSError;
pub use events::Letter;

use config::RsMixerConfig;
use events::{Dispatch, Message, Senders};

use std::{collections::HashMap, env, io::Write};

use tokio::runtime;
use tokio::sync::broadcast::channel;
use tokio::task;

use crossterm::{event::KeyCode, style::ContentStyle};

use log::LevelFilter;

use lazy_static::lazy_static;

use state::Storage;

lazy_static! {
    pub static ref DISPATCH: Dispatch<Letter> = Dispatch::new();
    pub static ref SENDERS: Senders<Letter> = Senders::new();
}
static STYLES: Storage<Styles> = Storage::new();
static BINDINGS: Storage<HashMap<KeyCode, Letter>> = Storage::new();

pub type Styles = HashMap<String, ContentStyle>;

async fn run() -> Result<(), RSError> {
    // @TODO choose where to log and verbosity
    let stdout = env::var("RUST_LOG").is_err();
    if stdout {
        simple_logging::log_to_file("log", LevelFilter::Debug).unwrap();
    } else {
        env_logger::init();
    }

    let config: RsMixerConfig = confy::load("rsmixer").unwrap();

    let (styles, bindings) = config.load();

    STYLES.set(styles);
    BINDINGS.set(bindings);

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

    let pa = async move {
        match task::spawn_blocking(move || pa::start(pa_rx)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let pa_async = async move {
        match task::spawn(async move { pa::start_async(pa_sx).await }).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    match tokio::try_join!(ui, pa, pa_async) {
        r => {
            DISPATCH.event(Letter::ExitSignal).await;

            let mut stdout = std::io::stdout();
            crossterm::execute!(
                stdout,
                crossterm::cursor::Show,
                crossterm::terminal::LeaveAlternateScreen
            )
            .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();

            if let Err(err) = r {
                println!("{}", err);
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), RSError> {
    let mut threaded_rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_time()
        .build()?;
    threaded_rt.block_on(async { run().await })
}
