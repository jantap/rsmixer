use thiserror::Error;

use super::PAInternal;

#[derive(Debug, Error)]
pub enum PAError {
	#[error("cannot create pulseaudio mainloop")]
	MainloopCreateError,
	#[error("cannot connect pulseaudio mainloop")]
	MainloopConnectError,
	#[error("cannot create pulseaudio stream")]
	StreamCreateError,
	#[error("internal channel send error")]
	ChannelError(#[from] cb_channel::SendError<PAInternal>),
	#[error("pulseaudio disconnected")]
	PulseAudioDisconnected,
}
