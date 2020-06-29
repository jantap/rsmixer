mod action_handlers;
mod draw;
mod models;
mod output;
mod util;
pub mod widgets;

use crate::input;
use crate::RSError;
use crate::{comms, SENDERS};
pub use util::{PageType, Rect};

use output::ui_loop;
use tokio::sync::broadcast::channel;

use tokio::task;

pub async fn start() -> Result<(), RSError> {
    let (sx, rx) = channel(comms::CHANNEL_CAPACITY);
    SENDERS.register(comms::UI_MESSAGE, sx).await;

    let w = async move {
        match task::spawn(input::start()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    let e = async move {
        match task::spawn(ui_loop(rx)).await {
            Ok(r) => r,
            Err(e) => Err(RSError::TaskHandleError(e)),
        }
    };

    tokio::try_join!(e, w)?;

    Ok(())
}
