use super::common::*;
use crate::{comms, SENDERS};
use std::time::Duration;
use tokio::stream::StreamExt;
use tokio::sync::broadcast::channel;
use tokio::sync::mpsc;

pub async fn start_async(internal_sx: cb_channel::Sender<PAInternal>) -> Result<(), RSError> {
    let (info_sx, mut info_rx) = mpsc::unbounded_channel();
    (*INFO_SX).set(info_sx);

    let (sx, mut command_receiver) = channel(comms::CHANNEL_CAPACITY);
    SENDERS.register(comms::PA_MESSAGE, sx).await;

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
        let inter = interval.next();

        tokio::select! {
            r = res => {
                if let Some(Ok(cmd)) = r {
                    if cmd == Letter::ExitSignal {
                        break;
                    }
                    internal_sx.send(PAInternal::Command(cmd))?;
                }
            }
            i = info => {
                if let Some(ident) = i {
                    send(&internal_sx, PAInternal::AskInfo(ident))?;
                }
            }
            _ = inter => {
                send(&internal_sx, PAInternal::Tick)?;
            }
        };
    }

    Ok(())
}
