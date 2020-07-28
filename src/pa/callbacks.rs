use super::common::*;

use crate::{
    entry::{Entry, EntrySpaceLvl},
    ui::widgets::VolumeWidget,
    DISPATCH,
};

use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo},
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
                is_selected: false,
                position: EntrySpaceLvl::Empty,
                volume_bar: VolumeWidget::default(),
                peak_volume_bar: VolumeWidget::default(),
                index: i.index,
                name,
                peak: 0.0,
                mute: i.mute,
                volume: i.volume,
                monitor_source: Some(i.monitor_source),
                parent: None,
                sink: None,
                suspended: i.state == SinkState::Suspended,
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
                is_selected: false,
                position: EntrySpaceLvl::Empty,
                volume_bar: VolumeWidget::default(),
                peak_volume_bar: VolumeWidget::default(),
                index: i.index,
                name: n,
                peak: 0.0,
                mute: i.mute,
                volume: i.volume,
                monitor_source: None,
                parent: Some(i.sink),
                sink: Some(i.sink),
                suspended: false,
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
                is_selected: false,
                position: EntrySpaceLvl::Empty,
                volume_bar: VolumeWidget::default(),
                peak_volume_bar: VolumeWidget::default(),
                index: i.index,
                name,
                peak: 0.0,
                mute: i.mute,
                volume: i.volume,
                monitor_source: Some(i.index),
                parent: None,
                sink: None,
                suspended: i.state == SourceState::Suspended,
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
                is_selected: false,
                position: EntrySpaceLvl::Empty,
                volume_bar: VolumeWidget::default(),
                peak_volume_bar: VolumeWidget::default(),
                index: i.index,
                name: n,
                peak: 0.0,
                mute: i.mute,
                volume: i.volume,
                monitor_source: Some(i.source),
                parent: Some(i.source),
                sink: None,
                suspended: false,
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
