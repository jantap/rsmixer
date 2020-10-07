use crate::{
    events::EventsManager,
    event_loop::event_loop,
    ui,
    RSError,
    DISPATCH,
    Letter,
    pa,
    events, input, SENDERS,
};

use std::future::Future;

use tokio::{
    sync::{
        mpsc,
        broadcast::channel,
    },
    task,
};


pub async fn run() -> Result<(), RSError> {
    let events = run_events().await;

    task::spawn(events);

    let event_loop = run_event_loop().await;
    let input_loop = run_input_loop();
    let (pa, pa_async) = run_pa().await;

    let pa_async = task::spawn(pa_async);
    let pa = task::spawn(pa);
    let input_loop = task::spawn(input_loop);
    let event_loop = task::spawn(event_loop);

    let r = tokio::try_join!(input_loop, pa, pa_async, event_loop);

    DISPATCH.event(Letter::ExitSignal).await;

    ui::clean_terminal()?;

    match r {
        Ok(_) => Ok(()),
        Err(err) => Ok(()),
    }
}

async fn run_events() -> impl Future<Output=Result<(), RSError>> {
    let ev_manager = EventsManager::prepare(&DISPATCH).await;

    async move { 
        match task::spawn(events::start(ev_manager, SENDERS.clone())).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
        
    }
}

async fn run_event_loop() -> impl Future<Output=Result<(), RSError>> {
    let (sx, rx) = channel(events::CHANNEL_CAPACITY);
    SENDERS.register(events::MAIN_MESSAGE, sx).await;

    async move {
        match task::spawn(event_loop(rx)).await {
            Ok(r) => r,
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    }
}

fn run_input_loop() -> impl Future<Output=Result<(), RSError>> {
    async move {
        match task::spawn(input::start()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    }
}

async fn run_pa() -> (impl Future<Output=Result<(), RSError>>, impl Future<Output=Result<(), RSError>>) {
    let (pa_sx, pa_rx) = cb_channel::unbounded();
    let (info_sx, info_rx) = mpsc::unbounded_channel();
    (*pa::INFO_SX).set(info_sx);

    let pa_async = async move {
        match task::spawn(async move { pa::start_async(pa_sx, info_rx).await }).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let pa = async move {
        match task::spawn_blocking(move || pa::start(pa_rx)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    (pa, pa_async)
}
