use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
	#[error("Config file is incorrect")]
	ConfyError(#[from] confy::ConfyError),
	#[error("'{0}' is not a valid key binding")]
	KeyCodeError(String),
	#[error("'{0}' is not a valid key action")]
	ActionBindingError(String),
	#[error("'{0}' is not a valid key color")]
	InvalidColor(String),
	#[error("'{0}' is not a valid key version code")]
	InvalidVersion(String),
}
