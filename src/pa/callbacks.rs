use super::common::*;

use crate::{
    entry::{Entry, CardProfile, PlayEntry, CardEntry, EntrySpaceLvl},
    ui::widgets::VolumeWidget,
    DISPATCH,
};

use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{CardInfo, SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo},
        subscribe::{subscription_masks, Operation},
    },
    def::{SinkState, SourceState},
};

pub fn subscribe(context: &Rc<RefCell<Context>>) -> Result<(), RSError> {
    info!("[PAInterface] Registering pulseaudio callbacks");

    context.borrow_mut().subscribe(
        subscription_masks::SINK
            | subscription_masks::SINK_INPUT
            | subscription_masks::SOURCE
            | subscription_masks::MASK_CARD
            | subscription_masks::SOURCE_OUTPUT,
        |success: bool| {
            assert!(success, "subscription failed");
        },
    );

    context.borrow_mut().set_subscribe_callback(Some(Box::new(
        move |facility, operation, index| {
            if let Some(facility) = facility {
                let entry_type: EntryType = facility.into();
                match operation {
                    Some(Operation::New) => {
                        info!("[PAInterface] New {:?}", entry_type);
                        (*INFO_SX)
                            .get()
                            .send(EntryIdentifier::new(entry_type, index))
                            .unwrap();
                    }
                    Some(Operation::Changed) => {
                        info!("[PAInterface] {:?} changed", entry_type);
                        (*INFO_SX)
                            .get()
                            .send(EntryIdentifier::new(entry_type, index))
                            .unwrap();
                    }
                    Some(Operation::Removed) => {
                        info!("[PAInterface] {:?} removed", entry_type);
                        DISPATCH.sync_event(Letter::EntryRemoved(EntryIdentifier::new(
                            entry_type, index,
                        )));
                    }
                    _ => {}
                };
            };
        },
    )));

    Ok(())
}

pub fn request_current_state(context: Rc<RefCell<Context>>) -> Result<(), RSError> {
    info!("[PAInterface] Requesting starting state");

    let introspector = context.borrow_mut().introspect();

    introspector.get_sink_info_list(|x: ListResult<&SinkInfo>| {
        if let ListResult::Item(e) = x {
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::Sink, e.index))
                .unwrap();
        }
    });

    introspector.get_sink_input_info_list(|x: ListResult<&SinkInputInfo>| {
        if let ListResult::Item(e) = x {
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::SinkInput, e.index))
                .unwrap();
        }
    });

    introspector.get_source_info_list(|x: ListResult<&SourceInfo>| {
        if let ListResult::Item(e) = x {
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::Source, e.index))
                .unwrap();
        }
    });

    introspector.get_source_output_info_list(|x: ListResult<&SourceOutputInfo>| {
        if let ListResult::Item(e) = x {
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::SourceOutput, e.index))
                .unwrap();
        }
    });

    introspector.get_card_info_list(|x: ListResult<&CardInfo>| {
        if let ListResult::Item(e) = x {
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::Card, e.index))
                .unwrap();
        }
    });

    Ok(())
}

pub fn request_info(ident: EntryIdentifier, context: &Rc<RefCell<Context>>) {
    let introspector = context.borrow_mut().introspect();
    debug!(
        "[PAInterface] Requesting info for {:?} {}",
        ident.entry_type, ident.index
    );
    match ident.entry_type {
        EntryType::SinkInput => {
            introspector.get_sink_input_info(ident.index, on_sink_input_info);
        }
        EntryType::Sink => {
            introspector.get_sink_info_by_index(ident.index, on_sink_info);
        }
        EntryType::SourceOutput => {
            introspector.get_source_output_info(ident.index, on_source_output_info);
        }
        EntryType::Source => {
            introspector.get_source_info_by_index(ident.index, on_source_info);
        }
        EntryType::Card => {
            introspector.get_card_info_by_index(ident.index, on_card_info);
        }
    };
}
pub fn on_card_info(res: ListResult<&CardInfo>) {
    match res {
        ListResult::Item(i) => {
            let n = match i
                .proplist
                .get_str(pulse::proplist::properties::DEVICE_DESCRIPTION)
            {
                Some(s) => s,
                None => String::from(""),
            };
            let profiles:Vec<CardProfile> = i.profiles.iter().filter_map(|p| { 
                if let Some(n) = &p.name {
                    Some(CardProfile {
                        name: n.to_string(),
                        description: match &p.description {
                            Some(s) => s.to_string(),
                            None => n.to_string(),
                        },
                        available: p.available,
                    })
                } else {
                    None
                }
            }).collect();

            let selected_profile = match &i.active_profile {
                Some(x) => {
                    if let Some(n) = &x.name {
                        profiles.iter().position(|p| 
                            p.name == n.to_string()
                        )
                    } else {
                        None
                    }
                }
                None => None,
            };

            let ident = EntryIdentifier::new(EntryType::Card, i.index);
            let entry = Entry {
                entry_type: EntryType::Card,
                index: i.index,
                name: n,
                parent: None,
                position: EntrySpaceLvl::Empty,
                is_selected: false,
                card_entry: Some(CardEntry { profiles, selected_profile }),
                play_entry:None,
            };

            DISPATCH.sync_event(Letter::EntryUpdate(ident, entry));
        }
        _ => {}
    };
}

pub fn on_sink_info(res: ListResult<&SinkInfo>) {
    match res {
        ListResult::Item(i) => {
            debug!("[PADataInterface] Update {} sink info", i.index);
            let name = match &i.description {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            let ident = EntryIdentifier::new(EntryType::Sink, i.index);
            let entry = Entry {
                entry_type: EntryType::Sink,
                index: i.index,
                name,
                parent: None,
                position: EntrySpaceLvl::Empty,
                is_selected: false,
                card_entry: None,
                play_entry: Some(PlayEntry {
                    volume_bar: VolumeWidget::default(),
                    peak_volume_bar: VolumeWidget::default(),
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: Some(i.monitor_source),
                    sink: None,
                    suspended: i.state == SinkState::Suspended,
                }),
            };
            DISPATCH.sync_event(Letter::EntryUpdate(ident, entry));
        }
        _ => {}
    };
}

pub fn on_sink_input_info(res: ListResult<&SinkInputInfo>) {
    match res {
        ListResult::Item(i) => {
            debug!("[PADataInterface] Update {} sink input info", i.index);
            let n = match i
                .proplist
                .get_str(pulse::proplist::properties::APPLICATION_NAME)
            {
                Some(s) => s,
                None => String::from(""),
            };
            let ident = EntryIdentifier::new(EntryType::SinkInput, i.index);
            let entry = Entry {
                entry_type: EntryType::SinkInput,
                parent: Some(i.sink),
                position: EntrySpaceLvl::Empty,
                name: n,
                index: i.index,
                is_selected: false,
                card_entry: None,
                play_entry: Some(PlayEntry {
                    volume_bar: VolumeWidget::default(),
                    peak_volume_bar: VolumeWidget::default(),
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: None,
                    sink: Some(i.sink),
                    suspended: false,
                }),
            };
            DISPATCH.sync_event(Letter::EntryUpdate(ident, entry));
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::Sink, i.sink))
                .unwrap();
        }
        _ => {}
    };
}

pub fn on_source_info(res: ListResult<&SourceInfo>) {
    match res {
        ListResult::Item(i) => {
            debug!("[PADataInterface] Update {} source info", i.index);
            let name = match &i.description {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            let ident = EntryIdentifier::new(EntryType::Source, i.index);
            let entry = Entry {
                entry_type: EntryType::Source,
                position: EntrySpaceLvl::Empty,
                index: i.index,
                name,
                parent: None,
                is_selected: false,
                card_entry: None,
                play_entry: Some(PlayEntry {
                    volume_bar: VolumeWidget::default(),
                    peak_volume_bar: VolumeWidget::default(),
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: Some(i.index),
                    sink: None,
                    suspended: i.state == SourceState::Suspended,
                }),
            };
            DISPATCH.sync_event(Letter::EntryUpdate(ident, entry));
        }
        _ => {}
    };
}

pub fn on_source_output_info(res: ListResult<&SourceOutputInfo>) {
    match res {
        ListResult::Item(i) => {
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
            let entry = Entry {
                entry_type: EntryType::SourceOutput,
                parent: Some(i.source),
                index: i.index,
                name: n,
                position: EntrySpaceLvl::Empty,
                is_selected: false,
                card_entry: None,
                play_entry: Some(PlayEntry {
                    volume_bar: VolumeWidget::default(),
                    peak_volume_bar: VolumeWidget::default(),
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: Some(i.source),
                    sink: None,
                    suspended: false,
                }),
            };
            DISPATCH.sync_event(Letter::EntryUpdate(ident, entry));
            (*INFO_SX)
                .get()
                .send(EntryIdentifier::new(EntryType::Source, i.index))
                .unwrap();
        }
        _ => {}
    };
}
