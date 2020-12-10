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
    let pa = run_pa().await;

    let pa = task::spawn(pa);
    let input_loop = task::spawn(input_loop);
    let event_loop = task::spawn(event_loop);

    let r = tokio::select! {
        res = input_loop => log::error!("KURWA SELECT1 {:?}", res), 
        res = pa => log::error!("KURWA SELECT2 {:?}", res), 
        res = event_loop => log::error!("KURWA SELECT3 {:?}", res), 
    };

    DISPATCH.event(Letter::ExitSignal).await;

    ui::clean_terminal()?;

    Ok(())
    // match r {
    //     Ok(_) => Ok(()),
    //     Err(err) => Ok(()),
    // }
}

async fn run_events() -> impl Future<Output=Result<(), RSError>> {
    let ev_manager = EventsManager::prepare(&DISPATCH).await;

    async move { 
        let r = match task::spawn(events::start(ev_manager, SENDERS.clone())).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        };
        log::error!("ENDENDEND events loop end");
        r
        
    }
}

async fn run_event_loop() -> impl Future<Output=Result<(), RSError>> {
    let (sx, rx) = channel(events::CHANNEL_CAPACITY);
    SENDERS.register(events::MAIN_MESSAGE, sx).await;

    async move {
        let r = match task::spawn(event_loop(rx)).await {
            Ok(r) => r,
            Err(e) => Err(RSError::TaskHandleError(e)),
        };
        log::error!("ENDENDEND eventslooploooplopy loop end");
        r
    }
}

fn run_input_loop() -> impl Future<Output=Result<(), RSError>> {
    async move {
        let r = match task::spawn(input::start()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        };
        log::error!("ENDENDEND input loop end");
        r
    }
}
async fn run_pa() -> impl Future<Output=Result<(), RSError>> {
    async move {
        let r = run_pa_internal().await;
        
        log::error!("ENDENDEND papapa");
        r
    }
}

async fn run_pa_internal() -> Result<(), RSError> {
    loop {
        let (pa_sx, pa_rx) = cb_channel::unbounded();
        let (info_sx, info_rx) = mpsc::unbounded_channel();


        let async_pa = task::spawn(async move { pa::start_async(pa_sx.clone(), info_rx).await });
        let sync_pa = task::spawn_blocking(move || pa::start(pa_rx, info_sx));

        let result = tokio::select! {
            res = async_pa => match res {
                Ok(r) => r,
                Err(e) => { return Err(RSError::TaskHandleError(e)); },
            },
            res = sync_pa => match res {
                Ok(r) => r,
                Err(e) => { return Err(RSError::TaskHandleError(e)); },
            },
        };

        log::error!("BROKEN DOWN {:?}", result);

        match result {
            Ok(value) => { break; },
            Err(_) => {} // retry
        }


        DISPATCH.event(Letter::PADisconnected).await;
        DISPATCH.event(Letter::PADisconnected2).await;

        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
    }

    Ok::<(), RSError>(())
}