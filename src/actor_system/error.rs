use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("message system Arc::try_unwrap failed. Perhaps you sent a message without using Context struct?")]
    WrongSystemMessage,
}
