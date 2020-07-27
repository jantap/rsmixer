use super::common::*;
use super::{callbacks, pa_actions};

use pulse::proplist::Proplist;
use std::ops::Deref;

pub fn start(internal_rx: cb_channel::Receiver<PAInternal>) -> Result<(), RSError> {
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
            return Err(RSError::MainloopCreateError);
        }
    }));

    debug!("[PAInterface] Creating new context");
    let context = Rc::new(RefCell::new(
        match Context::new_with_proplist(
            mainloop.borrow_mut().deref().deref(),
            "RsMixerContext",
            &proplist,
        ) {
            Some(ctx) => ctx,
            None => {
                error!("[PAInterface] Error while creating new context");
                return Err(RSError::MainloopCreateError);
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
            return Err(RSError::MainloopConnectError);
        }
    };

    info!("[PAInterface] Starting mainloop");

    // start mainloop
    mainloop.borrow_mut().lock();
    match mainloop.borrow_mut().start() {
        Ok(_) => {}
        Err(_) => {
            return Err(RSError::MainloopConnectError);
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
                return Err(RSError::MainloopConnectError);
            }
            _ => {
                mainloop.borrow_mut().wait();
            }
        }
    }
    debug!("[PAInterface] Context ready");

    context.borrow_mut().set_state_callback(None);

    callbacks::subscribe(&context)?;
    callbacks::request_current_state(Rc::clone(&context))?;

    mainloop.borrow_mut().unlock();

    debug!("[PAInterface] Actually starting our mainloop");

    let mut sink_input_monitors: Monitors = HashMap::new();
    let mut sink_monitors: Monitors = HashMap::new();
    let mut source_output_monitors: Monitors = HashMap::new();
    let mut source_monitors: Monitors = HashMap::new();


    while let Ok(msg) = internal_rx.recv() {
        mainloop.borrow_mut().lock();
        match msg {
            PAInternal::AskInfo(ident) => {
                callbacks::request_info(ident, &context);
            }
            PAInternal::Tick => {
                // remove failed monitors
                // sink_input_monitors.retain(pa_actions::remove_failed_monitors);
                // sink_monitors.retain(pa_actions::remove_failed_monitors);
                // source_output_monitors.retain(pa_actions::remove_failed_monitors);
                // source_monitors.retain(pa_actions::remove_failed_monitors);
            }
            PAInternal::Command(cmd) => {
                if let None = pa_actions::handle_command(cmd.clone(), &context) {
                    break;
                }

                if let Letter::CreateMonitors(monitors) = cmd.clone() {

                    pa_actions::create_monitors(
                        &mainloop,
                        &context,
                        &mut sink_monitors,
                        &mut sink_input_monitors,
                        &mut source_monitors,
                        &mut source_output_monitors,
                        &monitors,
                    );
                }
            }
        };
        mainloop.borrow_mut().unlock();
    }

    Ok(())
}
