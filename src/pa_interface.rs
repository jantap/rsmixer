use crate::{comms, pa_data_interface::*, Letter, Result, DISPATCH, SENDERS};

use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc, thread, time::Duration};

use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo},
        subscribe::{subscription_masks, Facility, Operation},
        Context,
    },
    mainloop::{
        api::Mainloop as MainloopTrait, //Needs to be in scope
        threaded::Mainloop,
    },
    proplist::Proplist,
    stream::Stream,
};

use async_std::future;
use async_std::prelude::*;
use async_std::sync::{channel, Receiver};
use async_std::task;
use lazy_static::lazy_static;
use log::{debug, error, info};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum PAError {
    #[error("Error while creating context and mainloop")]
    MainloopCreateError,
    #[error("Error while connecting to pulseaudio")]
    MainloopConnectError,
    #[error("Error while creating monitor stream")]
    StreamCreateError,
    #[error("Error while creating entry")]
    NoEntryError,
}

impl PAError {
    pub fn boxed<T>(&self) -> Result<T> {
        Err(Box::new((*self).clone()))
    }
}

lazy_static! {
    static ref SPEC: pulse::sample::Spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_FLOAT32,
        channels: 1,
        rate: 15,
    };
}

pub async fn start() -> Result<()> {
    let (sx, command_receiver) = channel(comms::CHANNEL_CAPACITY);
    SENDERS.register(comms::PA_MESSAGE, sx).await;

    // Create new mainloop and context
    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "RsMixer")
        .unwrap();

    debug!("[PAInterface] Creating new mainloop");
    let mainloop = Rc::new(RefCell::new(match Mainloop::new() {
        Some(ml) => ml,
        None => {
            error!("[PAInterface] Error while creating new mainloop");
            return PAError::MainloopCreateError.boxed();
        }
    }));

    debug!("[PAInterface] Creating new context");
    let context = Rc::new(RefCell::new(
        match Context::new_with_proplist(mainloop.borrow_mut().deref(), "RsMixerContext", &proplist)
        {
            Some(ctx) => ctx,
            None => {
                error!("[PAInterface] Error while creating new context");
                return PAError::MainloopCreateError.boxed();
            }
        },
    ));

    // Context state change callback
    {
        debug!("[PAInterface] Registering state change callback");
        let ml_ref = Rc::clone(&mainloop);
        let context_ref = Rc::clone(&context);
        context
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*context_ref.as_ptr()).get_state() };
                match state {
                    pulse::context::State::Ready
                    | pulse::context::State::Failed
                    | pulse::context::State::Terminated => {
                        unsafe { (*ml_ref.as_ptr()).signal(false) };
                    }
                    _ => {}
                }
            })));
    }

    // Try to connect to pulseaudio
    debug!("[PAInterface] Connecting context");

    match context
        .borrow_mut()
        .connect(None, pulse::context::flags::NOFLAGS, None)
    {
        Ok(_) => {}
        Err(_) => {
            error!("[PAInterface] Error while connecting context");
            return PAError::MainloopConnectError.boxed();
        }
    };

    info!("[PAInterface] Starting mainloop");

    // start mainloop
    mainloop.borrow_mut().lock();
    match mainloop.borrow_mut().start() {
        Ok(_) => {}
        Err(_) => {
            return PAError::MainloopConnectError.boxed();
        }
    }

    debug!("[PAInterface] Waiting for context to be ready...");
    // wait for context to be ready
    loop {
        match context.borrow_mut().get_state() {
            pulse::context::State::Ready => {
                break;
            }
            pulse::context::State::Failed | pulse::context::State::Terminated => {
                mainloop.borrow_mut().unlock();
                mainloop.borrow_mut().stop();
                error!("[PAInterface] Connection failed or context terminated");
                return PAError::MainloopConnectError.boxed();
            }
            _ => {
                mainloop.borrow_mut().wait();
            }
        }
    }
    debug!("[PAInterface] Context ready");

    context.borrow_mut().set_state_callback(None);

    register_callbacks(Rc::clone(&context))?;
    request_state(Rc::clone(&context))?;

    mainloop.borrow_mut().unlock();

    debug!("[PAInterface] Actually starting our mainloop");

    start_mainloop(Rc::clone(&mainloop), Rc::clone(&context), command_receiver).await?;

    Ok(())
}

async fn start_mainloop(
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    command_receiver: Receiver<Letter>,
) -> Result<()> {
    let mut sink_input_monitors: HashMap<u32, Rc<RefCell<Stream>>> = HashMap::new();
    let mut sink_monitors: HashMap<u32, Rc<RefCell<Stream>>> = HashMap::new();
    let mut source_output_monitors: HashMap<u32, Rc<RefCell<Stream>>> = HashMap::new();
    let mut source_monitors: HashMap<u32, Rc<RefCell<Stream>>> = HashMap::new();

    loop {
        let dur = Duration::from_millis(100);
        let res = command_receiver.recv().timeout(dur).await;

        if let Ok(cmd) = res {
            let cmd = cmd.unwrap();
            match cmd {
                Letter::MuteEntry(ident, mute) => {
                    match ident.entry_type {
                        EntryType::Sink => {
                            context.borrow_mut().introspect().set_sink_mute_by_index(
                                ident.index,
                                mute,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) = SINKS.lock().unwrap().get_mut(&ident.index) {
                                        s.mute = mute;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                        EntryType::SinkInput => {
                            context.borrow_mut().introspect().set_sink_input_mute(
                                ident.index,
                                mute,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) =
                                        SINK_INPUTS.lock().unwrap().get_mut(&ident.index)
                                    {
                                        s.mute = mute;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                        EntryType::Source => {
                            context.borrow_mut().introspect().set_source_mute_by_index(
                                ident.index,
                                mute,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) = SOURCES.lock().unwrap().get_mut(&ident.index) {
                                        s.mute = mute;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                        EntryType::SourceOutput => {
                            context.borrow_mut().introspect().set_source_output_mute(
                                ident.index,
                                mute,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) =
                                        SOURCE_OUTPUTS.lock().unwrap().get_mut(&ident.index)
                                    {
                                        s.mute = mute;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                    };
                }
                Letter::MoveEntryToParent(ident, parent) => {
                    match ident.entry_type {
                        EntryType::SinkInput => {
                            context.borrow_mut().introspect().move_sink_input_by_index(
                                ident.index,
                                parent.index,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    SINK_INPUTS.lock().unwrap().remove(&ident.index);
                                    // if let Some(s) =
                                    //     SINK_INPUTS.lock().unwrap().get_mut(&ident.index)
                                    // {
                                    //     s.sink = parent.index;
                                    //     task::spawn(async move {
                                    //         DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                    //         DISPATCH.event(Letter::EntryUpdate(parent)).await;
                                    //     });
                                    // }
                                    INFO_QUEUE.lock().unwrap().insert(parent);
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                        EntryType::SourceOutput => {
                            context.borrow_mut().introspect().move_source_output_by_index(
                                ident.index,
                                parent.index,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    SOURCE_OUTPUTS.lock().unwrap().remove(&ident.index);
                                    // if let Some(s) =
                                    //     SOURCE_OUTPUTS.lock().unwrap().get_mut(&ident.index)
                                    // {
                                    //     s.monitor_source = parent.index;
                                    //     task::spawn(async move {
                                    //         DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                    //         DISPATCH.event(Letter::EntryUpdate(parent)).await;
                                    //     });
                                    // }
                                    INFO_QUEUE.lock().unwrap().insert(parent);
                                    INFO_QUEUE.lock().unwrap().insert(ident);
                                })),
                            );
                        }
                        _ => {},
                    };
                }
                Letter::SetVolume(ident, vol) => {
                    match ident.entry_type {
                        EntryType::Sink => {
                            context.borrow_mut().introspect().set_sink_volume_by_index(
                                ident.index,
                                &vol,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) = SINKS.lock().unwrap().get_mut(&ident.index) {
                                        s.volume = vol;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                })),
                            );
                        }
                        EntryType::SinkInput => {
                            context.borrow_mut().introspect().set_sink_input_volume(
                                ident.index,
                                &vol,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) =
                                        SINK_INPUTS.lock().unwrap().get_mut(&ident.index)
                                    {
                                        s.volume = vol;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                })),
                            );
                        }
                        EntryType::Source => {
                            context
                                .borrow_mut()
                                .introspect()
                                .set_source_volume_by_index(
                                    ident.index,
                                    &vol,
                                    Some(Box::new(move |stat| {
                                        if !stat {
                                            return;
                                        }
                                        if let Some(s) =
                                            SOURCES.lock().unwrap().get_mut(&ident.index)
                                        {
                                            s.volume = vol;
                                            UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                            // task::spawn(async move {
                                            //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                            // });
                                        }
                                    })),
                                );
                        }
                        EntryType::SourceOutput => {
                            context.borrow_mut().introspect().set_source_output_volume(
                                ident.index,
                                &vol,
                                Some(Box::new(move |stat| {
                                    if !stat {
                                        return;
                                    }
                                    if let Some(s) =
                                        SOURCE_OUTPUTS.lock().unwrap().get_mut(&ident.index)
                                    {
                                        s.volume = vol;
                                        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().insert(ident);
                                        // task::spawn(async move {
                                        //     DISPATCH.event(Letter::EntryUpdate(ident)).await;
                                        // });
                                    }
                                })),
                            );
                        }
                        _ => {}
                    };
                }
                Letter::ExitSignal => {
                    //@TODO disconnect monitors
                    break;
                }
                _ => {}
            }
            continue;
        }

        mainloop.borrow_mut().lock();

        INFO_QUEUE.lock().unwrap().retain(|ident| {
            match ident.entry_type {
                EntryType::SinkInput => {
                    debug!(
                        "[PAInterface] Requesting info for sink input {}",
                        ident.index
                    );
                    context
                        .borrow_mut()
                        .introspect()
                        .get_sink_input_info(ident.index, on_sink_input_info);
                }
                EntryType::Sink => {
                    debug!("[PAInterface] Requesting info for sink {}", ident.index);
                    context
                        .borrow_mut()
                        .introspect()
                        .get_sink_info_by_index(ident.index, on_sink_info);
                }
                EntryType::SourceOutput => {
                    debug!(
                        "[PAInterface] Requesting info for source output {}",
                        ident.index
                    );
                    context
                        .borrow_mut()
                        .introspect()
                        .get_source_output_info(ident.index, on_source_output_info);
                }
                EntryType::Source => {
                    debug!("[PAInterface] Requesting info for source {}", ident.index);
                    context
                        .borrow_mut()
                        .introspect()
                        .get_source_info_by_index(ident.index, on_source_info);
                }
            };
            return false;
        });

        UI_UPDATE_ENTRY_QUEUE.lock().unwrap().retain(|ident| {
            let i = (*ident).clone();
            let e = match i.into_entry() {
                Ok(e) =>e,
                Err(_) => { return false;},
            };
            task::spawn(async move {;
                debug!("send ui update {:?} {}", i.entry_type, i.index);
                DISPATCH.event(Letter::EntryUpdate(i, e)).await;
            });
            return false;
        });

        debug!("[PAInterface] Checking if there are no unnecessary monitors");

        sink_input_monitors.retain(|index, monitor| {
            if !SINK_INPUTS.lock().unwrap().contains_key(index) {
                info!("[PAInterface] Disconnecting {} sink input monitor", index);
                return false;
            }
            match monitor.borrow_mut().get_state() {
                pulse::stream::State::Failed => {
                    info!(
                        "[PAInterface] Disconnecting {} sink input monitor (failed state)",
                        index
                    );
                    return false;
                }
                _ => true,
            }
        });

        sink_monitors.retain(|index, monitor| {
            if !SINKS.lock().unwrap().contains_key(index) {
                info!("[PAInterface] Disconnecting {} sink monitor", index);
                return false;
            }
            match monitor.borrow_mut().get_state() {
                pulse::stream::State::Failed => {
                    info!(
                        "[PAInterface] Disconnecting {} sink monitor (failed state)",
                        index
                    );
                    return false;
                }
                _ => true,
            }
        });

        source_output_monitors.retain(|index, monitor| {
            if !SOURCE_OUTPUTS.lock().unwrap().contains_key(index) {
                info!(
                    "[PAInterface] Disconnecting {} source output monitor",
                    index
                );
                return false;
            }
            match monitor.borrow_mut().get_state() {
                pulse::stream::State::Failed => {
                    info!(
                        "[PAInterface] Disconnecting {} source output monitor (failed state)",
                        index
                    );
                    return false;
                }
                _ => true,
            }
        });

        source_monitors.retain(|index, monitor| {
            if !SOURCES.lock().unwrap().contains_key(index) {
                info!("[PAInterface] Disconnecting {} source monitor", index);
                return false;
            }
            match monitor.borrow_mut().get_state() {
                pulse::stream::State::Failed => {
                    info!(
                        "[PAInterface] Disconnecting {} source monitor (failed state)",
                        index
                    );
                    return false;
                }
                _ => true,
            }
        });

        debug!("[PAInterface] Checking if we should create new monitors");

        for (index, sink_input) in SINK_INPUTS.lock().unwrap().iter() {
            if sink_input_monitors.contains_key(index) {
                continue;
            }

            info!("[PAInterface] Create new monitor for {} sink input", index);

            let monitor_src: u32;
            {
                match SINKS.lock().unwrap().get(&sink_input.sink) {
                    Some(sink) => monitor_src = sink.monitor_source,
                    None => continue,
                };
            }

            sink_input_monitors.insert(
                *index,
                create_monitor(
                    &mainloop,
                    &context,
                    &*SPEC,
                    EntryType::SinkInput,
                    Some(monitor_src),
                    Some(*index),
                )
                .unwrap(),
            );
        }

        for (index, sink) in SINKS.lock().unwrap().iter() {
            if sink_monitors.contains_key(index) {
                continue;
            }

            info!("[PAInterface] Create new monitor for {} sink", index);

            sink_monitors.insert(
                *index,
                create_monitor(
                    &mainloop,
                    &context,
                    &*SPEC,
                    EntryType::Sink,
                    Some(sink.monitor_source),
                    None,
                )
                .unwrap(),
            );
        }

        // for (index, source_output) in SOURCE_OUTPUTS.lock().unwrap().iter() {
        //     if source_output_monitors.contains_key(index) {
        //         continue;
        //     }

        //     info!("[PAInterface] Create new monitor for {} source output", index);

        //     source_output_monitors.insert(*index, create_monitor(Rc::clone(mainloop), Rc::clone(context), &spec, EntryType::SourceOutput, Some(source_output.monitor_source), None));
        // }

        for (index, source) in SOURCES.lock().unwrap().iter() {
            if source_monitors.contains_key(index) {
                continue;
            }

            info!("[PAInterface] Create new monitor for {} source", index);

            source_monitors.insert(
                *index,
                create_monitor(
                    &mainloop,
                    &context,
                    &*SPEC,
                    EntryType::Source,
                    Some(source.index),
                    None,
                )
                .unwrap(),
            );
        }

        mainloop.borrow_mut().unlock();
    }

    Ok(())
}

pub fn register_callbacks(context: Rc<RefCell<Context>>) -> Result<()> {
    info!("[PAInterface] Registering pulseaudio callbacks");

    context.borrow_mut().subscribe(
        subscription_masks::SINK
            | subscription_masks::SINK_INPUT
            | subscription_masks::SOURCE
            | subscription_masks::SOURCE_OUTPUT,
        |success: bool| {
            assert!(success, "subscription failed");
        },
    );

    context.borrow_mut().set_subscribe_callback(Some(Box::new(
        move |facility, operation, index| {
            match facility {
                Some(Facility::Source) => match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New source");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::Source, index));
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] Source changed");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::Source, index));
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] Source removed");
                        SOURCES.lock().unwrap().remove(&index);
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .remove(&EntryIdentifier::new(EntryType::Source, index));
                        task::spawn(async move {
                            DISPATCH
                                .event(Letter::EntryRemoved(EntryIdentifier::new(
                                    EntryType::Source,
                                    index,
                                )))
                                .await;
                        });
                    }
                    _ => {}
                },
                Some(Facility::SourceOutput) => match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New source output");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::SourceOutput, index));
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] Source output changed");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::SourceOutput, index));
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] Source output removed");
                        SOURCE_OUTPUTS.lock().unwrap().remove(&index);
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .remove(&EntryIdentifier::new(EntryType::SourceOutput, index));
                        task::spawn(async move {
                            DISPATCH
                                .event(Letter::EntryRemoved(EntryIdentifier::new(
                                    EntryType::SourceOutput,
                                    index,
                                )))
                                .await;
                        });
                    }
                    _ => {}
                },
                Some(Facility::Sink) => match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New sink");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::Sink, index));
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] Sink changed");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::Sink, index));
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] Sink removed");
                        SINKS.lock().unwrap().remove(&index);
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .remove(&EntryIdentifier::new(EntryType::Sink, index));
                        task::spawn(async move {
                            DISPATCH
                                .event(Letter::EntryRemoved(EntryIdentifier::new(
                                    EntryType::Sink,
                                    index,
                                )))
                                .await;
                        });
                    }
                    _ => {}
                },
                Some(Facility::SinkInput) => match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New sink input");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::SinkInput, index));
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] Sink input changed");
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .insert(EntryIdentifier::new(EntryType::SinkInput, index));
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] Sink input removed");
                        SINK_INPUTS.lock().unwrap().remove(&index);
                        INFO_QUEUE
                            .lock()
                            .unwrap()
                            .remove(&EntryIdentifier::new(EntryType::SinkInput, index));
                        task::spawn(async move {
                            DISPATCH
                                .event(Letter::EntryRemoved(EntryIdentifier::new(
                                    EntryType::SinkInput,
                                    index,
                                )))
                                .await;
                        });
                    }
                    _ => {}
                },
                _ => {}
            };
        },
    )));

    Ok(())
}

pub fn request_state(context: Rc<RefCell<Context>>) -> Result<()> {
    info!("[PAInterface] Requesting starting state");

    context
        .borrow_mut()
        .introspect()
        .get_sink_info_list(|x: ListResult<&SinkInfo>| {
            if let ListResult::Item(e) = x {
                INFO_QUEUE
                    .lock()
                    .unwrap()
                    .insert(EntryIdentifier::new(EntryType::Sink, e.index));
            }
        });

    context
        .borrow_mut()
        .introspect()
        .get_sink_input_info_list(|x: ListResult<&SinkInputInfo>| {
            if let ListResult::Item(e) = x {
                INFO_QUEUE
                    .lock()
                    .unwrap()
                    .insert(EntryIdentifier::new(EntryType::SinkInput, e.index));
            }
        });

    context
        .borrow_mut()
        .introspect()
        .get_source_info_list(|x: ListResult<&SourceInfo>| {
            if let ListResult::Item(e) = x {
                INFO_QUEUE
                    .lock()
                    .unwrap()
                    .insert(EntryIdentifier::new(EntryType::Source, e.index));
            }
        });

    context
        .borrow_mut()
        .introspect()
        .get_source_output_info_list(|x: ListResult<&SourceOutputInfo>| {
            if let ListResult::Item(e) = x {
                INFO_QUEUE
                    .lock()
                    .unwrap()
                    .insert(EntryIdentifier::new(EntryType::SourceOutput, e.index));
            }
        });

    Ok(())
}
