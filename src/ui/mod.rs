mod action_handlers;
mod draw;
mod models;
mod output;
mod util;
pub mod widgets;

pub use util::{PageType, Rect};

use output::ui_loop;

use crate::{events, input, RSError, SENDERS};

use tokio::{sync::broadcast::channel, task};

pub async fn start() -> Result<(), RSError> {
    let (sx, rx) = channel(events::CHANNEL_CAPACITY);
    SENDERS.register(events::UI_MESSAGE, sx).await;

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
