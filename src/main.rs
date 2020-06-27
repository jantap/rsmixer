#![feature(const_fn)]

extern crate libpulse_binding as pulse;

mod bishop;
mod comms;
mod input;
mod pa_data_interface;
mod pa_interface;
mod ui;

use pa_data_interface::*;

use bishop::{BishopMessage, Dispatch, Senders};
use std::collections::HashMap;
use std::env;
use ui::Entry;
use ui::PageType;

use async_std::task;
use log::LevelFilter;

use async_std::prelude::*;

use async_std::sync::channel;

use comms::Letter;
use lazy_static::lazy_static;

use crossterm::event::KeyCode;
use crossterm::event::KeyCode::Char;
use crossterm::style::Attribute;
use crossterm::style::Color;
use crossterm::style::ContentStyle;

use state::Storage;

lazy_static! {
    static ref DISPATCH: Dispatch<Letter> = Dispatch::new();
    static ref SENDERS: Senders<Letter> = Senders::new();
}
static STYLES: Storage<Styles> = Storage::new();
static BINDINGS: Storage<HashMap<KeyCode, Letter>> = Storage::new();
static CONTEXT_MENUS: Storage<HashMap<EntryType, Vec<(&'static str, Letter)>>> = Storage::new();

pub type Styles = HashMap<&'static str, ContentStyle>;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn run() -> Result<()> {
    let stdout = env::var("RUST_LOG").is_err();
    if stdout {
        simple_logging::log_to_file("log", LevelFilter::Info).unwrap();
    } else {
        env_logger::init();
    }

    let mut bindings = HashMap::new();
    bindings.insert(Char('q'), Letter::ExitSignal);
    bindings.insert(Char('j'), Letter::MoveDown(1));
    bindings.insert(Char('k'), Letter::MoveUp(1));
    bindings.insert(Char('m'), Letter::RequestMute);
    bindings.insert(Char('h'), Letter::RequstChangeVolume(-5));
    bindings.insert(Char('l'), Letter::RequstChangeVolume(5));
    bindings.insert(Char('H'), Letter::RequstChangeVolume(-15));
    bindings.insert(Char('L'), Letter::RequstChangeVolume(15));
    bindings.insert(Char('1'), Letter::ChangePage(PageType::Output));
    bindings.insert(Char('2'), Letter::ChangePage(PageType::Input));
    bindings.insert(crossterm::event::KeyCode::Enter, Letter::OpenContextMenu);
    BINDINGS.set(bindings);

    let mut styles: Styles = HashMap::new();
    styles.insert(
        "normal",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::White),
    );
    styles.insert(
        "inverted",
        ContentStyle::new()
            .background(Color::White)
            .foreground(Color::Black),
    );
    styles.insert(
        "muted",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::Grey),
    );
    styles.insert(
        "bold",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::White)
            .attribute(Attribute::Bold),
    );
    styles.insert(
        "red",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::Red),
    );
    styles.insert(
        "orange",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::Yellow),
    );
    styles.insert(
        "green",
        ContentStyle::new()
            .background(Color::Black)
            .foreground(Color::Green),
    );
    styles.insert(
        "test",
        ContentStyle::new()
            .background(Color::DarkBlue)
            .foreground(Color::Red),
    );

    STYLES.set(styles);

    let (event_sx, event_rx) = channel(32);
    DISPATCH.register(event_sx).await;

    let events = task::spawn(async move {
        bishop::start(event_rx, SENDERS.clone()).await;
    });

    let ui = task::spawn(async move {
        ui::start().await.unwrap();
    });

    // let pa_future = task::spawn(async move {
    //     pa_interface::start().await.unwrap();

    //     // task::block_on(pa_interface.connect_and_start_mainloop()).unwrap();
    // });

    let x = events.join(ui).join(pa_interface::start());
    log::error!("start program");
    x.await;
    log::error!("quit program");
    Ok(())

    // let chan_for_ui = (send_data_to_input.clone(), ui_recv_orders.clone());
    // let ui_thread = thread::spawn(move || {
    //     let stdout = io::stdout();
    //     let stdout = stdout.lock();
    //     let mut stdout = termion::cursor::HideCursor::from(stdout.into_raw_mode().unwrap());

    //     let mut ui = UI::new(stdout, chan_for_ui);
    //     ui.start_ui();
    // });

    // let input = Input::new((send_orders_to_ui, recv_data_in_input), send_orders_to_pa);
    // input.start_input(stdin);

    // ui_thread.join().unwrap();
    // pa_interface_thread.join().unwrap();
}

fn main() -> Result<()> {
    task::block_on(run())
}
