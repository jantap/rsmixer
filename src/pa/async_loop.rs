use super::common::*;

use crate::{events, SENDERS};

use std::time::Duration;

use tokio::{
    stream::StreamExt,
    sync::{broadcast::channel, mpsc},
};

pub async fn start_async(
    internal_sx: cb_channel::Sender<PAInternal>,
    mut info_rx: mpsc::UnboundedReceiver<EntryIdentifier>,
) -> Result<(), RSError> {
    let (sx, mut command_receiver) = channel(events::CHANNEL_CAPACITY);
    SENDERS.register(events::PA_MESSAGE, sx).await;

    let mut interval = tokio::time::interval(Duration::from_millis(50));

    let send = |ch: &cb_channel::Sender<PAInternal>, msg: PAInternal| -> Result<(), RSError> {
        match ch.send(msg) {
            Ok(()) => Ok(()),
            Err(err) => Err(RSError::ChannelError(err)),
        }
    };

    loop {
        let res = command_receiver.next();
        let info = info_rx.next();
        let timeout = interval.next();

        tokio::select! {
            r = res => {
                if let Some(Ok(cmd)) = r {
                    if let Action::ExitSignal = cmd {
                        break;
                    }
                    internal_sx.send(PAInternal::Command(Box::new(cmd)))?;
                }
            }
            i = info => {
                if let Some(ident) = i {
                    send(&internal_sx, PAInternal::AskInfo(ident))?;
                }
            }
            _ = timeout => {
                send(&internal_sx, PAInternal::Tick)?;
            }
        };
    }

    Ok(())
}
