use std::ops::Deref;

use lazy_static::lazy_static;
use pulse::proplist::Proplist;
use state::Storage;

use super::{callbacks, common::*, pa_actions};

lazy_static! {
	pub static ref ACTIONS_SX: Storage<mpsc::UnboundedSender<EntryUpdate>> = Storage::new();
}

pub fn start(
	internal_rx: cb_channel::Receiver<PAInternal>,
	info_sx: mpsc::UnboundedSender<EntryIdentifier>,
	actions_sx: mpsc::UnboundedSender<EntryUpdate>,
) -> Result<()> {
	(*ACTIONS_SX).set(actions_sx);

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
			return Err(PAError::MainloopCreateError.into());
		}
	}));

	debug!("[PAInterface] Creating new context");
	let context = Rc::new(RefCell::new(
		match PAContext::new_with_proplist(
			mainloop.borrow_mut().deref().deref(),
			"RsMixerContext",
			&proplist,
		) {
			Some(ctx) => ctx,
			None => {
				error!("[PAInterface] Error while creating new context");
				return Err(PAError::MainloopCreateError.into());
			}
		},
	));

	// PAContext state change callback
	{
		debug!("[PAInterface] Registering state change callback");
		let ml_ref = Rc::clone(&mainloop);
		let context_ref = Rc::clone(&context);
		context
			.borrow_mut()
			.set_state_callback(Some(Box::new(move || {
				let state = unsafe { (*context_ref.as_ptr()).get_state() };
				if matches!(
					state,
					pulse::context::State::Ready
						| pulse::context::State::Failed
						| pulse::context::State::Terminated
				) {
					unsafe { (*ml_ref.as_ptr()).signal(false) };
				}
			})));
	}

	// Try to connect to pulseaudio
	debug!("[PAInterface] Connecting context");

	if context
		.borrow_mut()
		.connect(None, pulse::context::FlagSet::NOFLAGS, None)
		.is_err()
	{
		error!("[PAInterface] Error while connecting context");
		return Err(PAError::MainloopConnectError.into());
	}

	info!("[PAInterface] Starting mainloop");

	// start mainloop
	mainloop.borrow_mut().lock();
	if mainloop.borrow_mut().start().is_err() {
		return Err(PAError::MainloopConnectError.into());
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
				return Err(PAError::MainloopConnectError.into());
			}
			_ => {
				mainloop.borrow_mut().wait();
			}
		}
	}
	debug!("[PAInterface] PAContext ready");

	context.borrow_mut().set_state_callback(None);

	callbacks::subscribe(&context, info_sx.clone())?;
	callbacks::request_current_state(context.clone(), info_sx.clone())?;

	mainloop.borrow_mut().unlock();

	debug!("[PAInterface] Actually starting our mainloop");

	let mut monitors = Monitors::default();
	let mut last_targets = HashMap::new();

	while let Ok(msg) = internal_rx.recv() {
		mainloop.borrow_mut().lock();

		match context.borrow_mut().get_state() {
			pulse::context::State::Ready => {}
			_ => {
				mainloop.borrow_mut().unlock();
				return Err(PAError::PulseAudioDisconnected).context("disconnected while working");
			}
		}

		match msg {
			PAInternal::AskInfo(ident) => {
				callbacks::request_info(ident, &context, info_sx.clone());
			}
			PAInternal::Tick => {
				// remove failed monitors
				monitors.filter(&mainloop, &context, &last_targets);
			}
			PAInternal::Command(cmd) => {
				let cmd = cmd.deref();
				if pa_actions::handle_command(cmd.clone(), &context, &info_sx).is_none() {
					monitors.filter(&mainloop, &context, &HashMap::new());
					mainloop.borrow_mut().unlock();
					break;
				}

				if let PulseAudioAction::CreateMonitors(mons) = cmd.clone() {
					last_targets = mons;
					monitors.filter(&mainloop, &context, &last_targets);
				}
			}
		};
		mainloop.borrow_mut().unlock();
	}

	Ok(())
}
