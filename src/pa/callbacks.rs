use super::{common::*, sync_loop::ACTIONS_SX};

use crate::{models::EntryUpdate, entry::{CardProfile, Entry}};

use crate::ui::Rect;

use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{CardInfo, SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo},
        subscribe::{InterestMaskSet, Operation},
    },
    def::{SinkState, SourceState},
};

pub fn subscribe(
    context: &Rc<RefCell<Context>>,
    info_sx: mpsc::UnboundedSender<EntryIdentifier>,
) -> Result<(), RsError> {
    info!("[PAInterface] Registering pulseaudio callbacks");

    context.borrow_mut().subscribe(
        InterestMaskSet::SINK
            | InterestMaskSet::SINK_INPUT
            | InterestMaskSet::SOURCE
            | InterestMaskSet::CARD
            | InterestMaskSet::SOURCE_OUTPUT
            | InterestMaskSet::CLIENT
            | InterestMaskSet::SERVER,
        |success: bool| {
            assert!(success, "subscription failed");
        },
    );

    context.borrow_mut().set_subscribe_callback(Some(Box::new(
        move |facility, operation, index| {
            if let Some(facility) = facility {
                match facility {
                    Facility::Server | Facility::Client => {
                        log::error!("{:?} {:?}", facility, operation);
                        return;
                    }
                    _ => {}
                };

                let entry_type: EntryType = facility.into();
                match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New {:?}", entry_type);

                        info_sx
                            .send(EntryIdentifier::new(entry_type, index))
                            .unwrap();
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] {:?} changed", entry_type);
                        info_sx
                            .send(EntryIdentifier::new(entry_type, index))
                            .unwrap();
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] {:?} removed", entry_type);
                        (*ACTIONS_SX)
                            .get()
                            .send(EntryUpdate::EntryRemoved(EntryIdentifier::new(
                                entry_type, index,
                            )))
                            .unwrap();
                    }
                    _ => {}
                };
            };
        },
    )));

    Ok(())
}

pub fn request_current_state(
    context: Rc<RefCell<Context>>,
    info_sxx: mpsc::UnboundedSender<EntryIdentifier>,
) -> Result<(), RsError> {
    info!("[PAInterface] Requesting starting state");

    let introspector = context.borrow_mut().introspect();

    let info_sx = info_sxx.clone();
    introspector.get_sink_info_list(move |x: ListResult<&SinkInfo>| {
        if let ListResult::Item(e) = x {
            let _ = info_sx
                .clone()
                .send(EntryIdentifier::new(EntryType::Sink, e.index));
        }
    });

    let info_sx = info_sxx.clone();
    introspector.get_sink_input_info_list(move |x: ListResult<&SinkInputInfo>| {
        if let ListResult::Item(e) = x {
            let _ = info_sx.send(EntryIdentifier::new(EntryType::SinkInput, e.index));
        }
    });

    let info_sx = info_sxx.clone();
    introspector.get_source_info_list(move |x: ListResult<&SourceInfo>| {
        if let ListResult::Item(e) = x {
            let _ = info_sx.send(EntryIdentifier::new(EntryType::Source, e.index));
        }
    });

    let info_sx = info_sxx.clone();
    introspector.get_source_output_info_list(move |x: ListResult<&SourceOutputInfo>| {
        if let ListResult::Item(e) = x {
            let _ = info_sx.send(EntryIdentifier::new(EntryType::SourceOutput, e.index));
        }
    });

    introspector.get_card_info_list(move |x: ListResult<&CardInfo>| {
        if let ListResult::Item(e) = x {
            let _ = info_sxx.send(EntryIdentifier::new(EntryType::Card, e.index));
        }
    });

    Ok(())
}

pub fn request_info(
    ident: EntryIdentifier,
    context: &Rc<RefCell<Context>>,
    info_sx: mpsc::UnboundedSender<EntryIdentifier>,
) {
    let introspector = context.borrow_mut().introspect();
    debug!(
        "[PAInterface] Requesting info for {:?} {}",
        ident.entry_type, ident.index
    );
    match ident.entry_type {
        EntryType::SinkInput => {
            introspector.get_sink_input_info(ident.index, on_sink_input_info(&info_sx));
        }
        EntryType::Sink => {
            introspector.get_sink_info_by_index(ident.index, on_sink_info(&info_sx));
        }
        EntryType::SourceOutput => {
            introspector.get_source_output_info(ident.index, on_source_output_info(&info_sx));
        }
        EntryType::Source => {
            introspector.get_source_info_by_index(ident.index, on_source_info(&info_sx));
        }
        EntryType::Card => {
            introspector.get_card_info_by_index(ident.index, on_card_info);
        }
    };
}
pub fn on_card_info(res: ListResult<&CardInfo>) {
    if let ListResult::Item(i) = res {
        let n = match i
            .proplist
            .get_str(pulse::proplist::properties::DEVICE_DESCRIPTION)
        {
            Some(s) => s,
            None => String::from(""),
        };
        let profiles: Vec<CardProfile> = i
            .profiles
            .iter()
            .filter_map(|p| {
                if let Some(n) = &p.name {
                    Some(CardProfile {
                        area: Rect::default(),
                        is_selected: false,
                        name: n.to_string(),
                        description: match &p.description {
                            Some(s) => s.to_string(),
                            None => n.to_string(),
                        },
                        #[cfg(any(feature = "pa_v13"))]
                        available: p.available,
                    })
                } else {
                    None
                }
            })
            .collect();

        let selected_profile = match &i.active_profile {
            Some(x) => {
                if let Some(n) = &x.name {
                    profiles.iter().position(|p| p.name == *n)
                } else {
                    None
                }
            }
            None => None,
        };

        let ident = EntryIdentifier::new(EntryType::Card, i.index);
        let entry = Entry::new_card_entry(i.index, n, profiles, selected_profile);

        (*ACTIONS_SX)
            .get()
            .send(EntryUpdate::EntryUpdate(ident, Box::new(entry)))
            .unwrap();
    }
}

pub fn on_sink_info(
    _sx: &mpsc::UnboundedSender<EntryIdentifier>,
) -> impl Fn(ListResult<&SinkInfo>) {
    |res: ListResult<&SinkInfo>| {
        if let ListResult::Item(i) = res {
            debug!("[PADataInterface] Update {} sink info", i.index);
            let name = match &i.description {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            let ident = EntryIdentifier::new(EntryType::Sink, i.index);
            let entry = Entry::new_play_entry(
                EntryType::Sink,
                i.index,
                name,
                None,
                i.mute,
                i.volume,
                Some(i.monitor_source),
                None,
                i.state == SinkState::Suspended,
            );

            (*ACTIONS_SX)
                .get()
                .send(EntryUpdate::EntryUpdate(ident, Box::new(entry)))
                .unwrap();
        }
    }
}

pub fn on_sink_input_info(
    sx: &mpsc::UnboundedSender<EntryIdentifier>,
) -> impl Fn(ListResult<&SinkInputInfo>) {
    let info_sx = sx.clone();
    move |res: ListResult<&SinkInputInfo>| {
        if let ListResult::Item(i) = res {
            debug!("[PADataInterface] Update {} sink input info", i.index);
            let n = match i
                .proplist
                .get_str(pulse::proplist::properties::APPLICATION_NAME)
            {
                Some(s) => s,
                None => match &i.name {
                    Some(s) => s.to_string(),
                    None => String::from(""),
                },
            };
            let ident = EntryIdentifier::new(EntryType::SinkInput, i.index);

            let entry = Entry::new_play_entry(
                EntryType::SinkInput,
                i.index,
                n,
                Some(i.sink),
                i.mute,
                i.volume,
                None,
                Some(i.sink),
                false,
            );

            (*ACTIONS_SX)
                .get()
                .send(EntryUpdate::EntryUpdate(ident, Box::new(entry)))
                .unwrap();
            let _ = info_sx.send(EntryIdentifier::new(EntryType::Sink, i.sink));
        }
    }
}

pub fn on_source_info(
    _sx: &mpsc::UnboundedSender<EntryIdentifier>,
) -> impl Fn(ListResult<&SourceInfo>) {
    move |res: ListResult<&SourceInfo>| {
        if let ListResult::Item(i) = res {
            debug!("[PADataInterface] Update {} source info", i.index);
            let name = match &i.description {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            let ident = EntryIdentifier::new(EntryType::Source, i.index);
            let entry = Entry::new_play_entry(
                EntryType::Source,
                i.index,
                name,
                None,
                i.mute,
                i.volume,
                Some(i.index),
                None,
                i.state == SourceState::Suspended,
            );

            (*ACTIONS_SX)
                .get()
                .send(EntryUpdate::EntryUpdate(ident, Box::new(entry)))
                .unwrap();
        }
    }
}

pub fn on_source_output_info(
    sx: &mpsc::UnboundedSender<EntryIdentifier>,
) -> impl Fn(ListResult<&SourceOutputInfo>) {
    let info_sx = sx.clone();
    move |res: ListResult<&SourceOutputInfo>| {
        if let ListResult::Item(i) = res {
            debug!("[PADataInterface] Update {} source output info", i.index);
            let n = match i
                .proplist
                .get_str(pulse::proplist::properties::APPLICATION_NAME)
            {
                Some(s) => s,
                None => String::from(""),
            };
            if n == "RsMixerContext" {
                return;
            }
            let ident = EntryIdentifier::new(EntryType::SourceOutput, i.index);
            let entry = Entry::new_play_entry(
                EntryType::SourceOutput,
                i.index,
                n,
                Some(i.source),
                i.mute,
                i.volume,
                Some(i.source),
                None,
                false,
            );

            (*ACTIONS_SX)
                .get()
                .send(EntryUpdate::EntryUpdate(ident, Box::new(entry)))
                .unwrap();
            let _ = info_sx.send(EntryIdentifier::new(EntryType::Source, i.index));
        }
    }
}
