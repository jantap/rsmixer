use std::{error::Error, fmt};

use thiserror::Error;

use crate::pa::PAInternal;

#[derive(Debug)]
pub enum RsError {
	// General errors
	TaskHandleError(tokio::task::JoinError),

	// Config related errors
	ConfyError(confy::ConfyError),
	KeyCodeError(String),
	ActionBindingError(String),
	InvalidColor(String),
	InvalidVersion(String),

	// UI related errors
	TerminalTooSmall,
	TerminalError(crossterm::ErrorKind),
	IoError(std::io::Error),

	// PulseAudio related errors
	MainloopCreateError,
	MainloopConnectError,
	StreamCreateError,
	ChannelError(cb_channel::SendError<PAInternal>),
	NoEntryError,
	PulseAudioDisconnected,
}

impl Error for RsError {}

impl fmt::Display for RsError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::TaskHandleError(e) => {
				log::error!("{:#}", e);
				write!(f, "Join handle error\n{:#}", e)
			}
			Self::TerminalError(_) => write!(f, "Terminal error"),
			Self::TerminalTooSmall => write!(f, "Terminal is too small"),
			Self::IoError(_) => write!(f, "IO error"),
			Self::MainloopCreateError => write!(f, "Error while creating context and mainloop"),
			Self::MainloopConnectError => write!(f, "Error while connecting to pulseaudio"),
			Self::StreamCreateError => write!(f, "Error while creating monitor stream"),
			Self::PulseAudioDisconnected => write!(f, "PulseAudio disconnected"),
			Self::ChannelError(_) => write!(
				f,
				"Error in internal communication between pulseaudio threads"
			),
			Self::NoEntryError => write!(f, "Error while creating entry"),
			Self::ConfyError(err) => {
				write!(f, "{:#}\n\nIf you've just updated RsMixer version see", err)
			}
			Self::KeyCodeError(kc) => write!(
				f,
				"Error in config file\n'{}' is not a valid key binding",
				kc
			),
			Self::ActionBindingError(act) => {
				write!(f, "Error in config file\n'{}' is not a valid action", act)
			}
			Self::InvalidColor(color) => {
				write!(f, "Error in config file\n'{}' is not a valid color", color)
			}
			Self::InvalidVersion(version) => {
				write!(
					f,
					"Error in config file\n'{}' is not a valid version code",
					version
				)
			}
		}
	}
}

impl From<std::io::Error> for RsError {
	fn from(err: std::io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<crossterm::ErrorKind> for RsError {
	fn from(err: crossterm::ErrorKind) -> Self {
		Self::TerminalError(err)
	}
}

impl From<cb_channel::SendError<PAInternal>> for RsError {
	fn from(chan: cb_channel::SendError<PAInternal>) -> RsError {
		RsError::ChannelError(chan)
	}
}

impl From<tokio::task::JoinError> for RsError {
	fn from(err: tokio::task::JoinError) -> RsError {
		RsError::TaskHandleError(err)
	}
}

impl From<confy::ConfyError> for RsError {
	fn from(err: confy::ConfyError) -> RsError {
		RsError::ConfyError(err)
	}
}

impl From<semver::SemVerError> for RsError {
	fn from(_err: semver::SemVerError) -> RsError {
		RsError::InvalidVersion("".to_string())
	}
}
