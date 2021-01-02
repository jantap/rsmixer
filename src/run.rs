use crate::{
    event_loop::event_loop, input, models::actions::statics::*, pa, ui, Action, RSError, DISPATCH,
    SENDERS, VARIABLES,
};

use ev_apple::EventsManager;

use std::future::Future;

use tokio::{
    stream::StreamExt,
    sync::{broadcast::channel, mpsc},
    task,
};

pub async fn run() -> Result<(), RSError> {
    let events = run_events().await;

    task::spawn(events);

    let event_loop = run_event_loop().await;
    let input_loop = run_input_loop();
    let pa = run_pa().await;

    let pa = task::spawn(pa);
    let input_loop = task::spawn(input_loop);
    let event_loop = task::spawn(event_loop);

    let r = tokio::try_join!(input_loop, pa, event_loop);

    DISPATCH.event(Action::ExitSignal).await;

    ui::clean_terminal()?;

    match r {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

async fn run_events() -> impl Future<Output = Result<(), RSError>> {
    let ev_manager = EventsManager::prepare(&DISPATCH, EXIT_MESSAGE_ID).await;

    async move {
        match task::spawn(ev_apple::start(ev_manager, SENDERS.clone())).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    }
}

async fn run_event_loop() -> impl Future<Output = Result<(), RSError>> {
    let (sx, rx) = channel(CHANNEL_CAPACITY);
    SENDERS.register(MAIN_MESSAGE, sx).await;

    async move {
        match task::spawn(event_loop(rx)).await {
            Ok(r) => r,
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    }
}

async fn run_input_loop() -> Result<(), RSError> {
    let (sx, rx) = channel(CHANNEL_CAPACITY);
    SENDERS.register(INPUT_MESSAGE, sx).await;

    match task::spawn(input::start(rx)).await {
        Ok(_) => Ok(()),
        Err(e) => Err(RSError::TaskHandleError(e)),
    }
}
async fn run_pa() -> impl Future<Output = Result<(), RSError>> {
    async move { run_pa_internal().await }
}

async fn run_pa_internal() -> Result<(), RSError> {
    let (sx, mut rx) = channel(32);
    SENDERS.register(RUN_PA_MESSAGE, sx).await;

    let retry_time = (*VARIABLES).get().pa_retry_time;

    loop {
        let (pa_sx, pa_rx) = cb_channel::unbounded();
        let (info_sx, info_rx) = mpsc::unbounded_channel();

        let async_pa = task::spawn(async move { pa::start_async(pa_sx.clone(), info_rx).await });
        let sync_pa = task::spawn_blocking(move || pa::start(pa_rx, info_sx));
        DISPATCH.event(Action::ConnectToPA).await;

        let result = tokio::select! {
            res = async_pa => match res {
                Ok(r) => r,
                Err(e) => { return Err(RSError::TaskHandleError(e)); },
            },
            res = sync_pa => {
                match res {
                    Ok(r) => r,
                    Err(e) => { return Err(RSError::TaskHandleError(e)); },
                }
            },
        };

        if result.is_ok() {
            break;
        }

        DISPATCH.event(Action::PADisconnected).await;
        DISPATCH.event(Action::PADisconnected2).await;

        for i in 0..retry_time {
            DISPATCH.event(Action::RetryIn(retry_time - i)).await;

            let timeout_part = tokio::time::delay_for(std::time::Duration::from_secs(1));
            let event = rx.next();
            tokio::select! {
                _ = timeout_part => {},
                ev = event => {
                    if let Some(Ok(Action::ExitSignal)) = ev {
                        return Ok(());
                    }
                }
            };
        }
    }

    Ok::<(), RSError>(())
}
