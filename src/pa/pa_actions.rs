use super::common::*;
use super::monitor;
use crate::entry::Entry;

pub fn handle_command(cmd: Letter, context: &Rc<RefCell<Context>>) -> Option<()> {
    match cmd {
        Letter::MuteEntry(ident, mute) => {
            set_mute(ident, mute, &context);
        }
        Letter::MoveEntryToParent(ident, parent) => {
            move_entry_to_parent(ident, parent, &context);
        }
        Letter::SetVolume(ident, vol) => {
            set_volume(ident, vol, &context);
        }
        Letter::SetSuspend(ident, suspend) => {
            set_suspend(ident, suspend, &context);
        }
        Letter::KillEntry(ident) => {
            kill_entry(ident, &context);
        }
        Letter::ExitSignal => {
            //@TODO disconnect monitors
            return None;
        }
        _ => {}
    };
    Some(())
}

fn set_volume(
    ident: EntryIdentifier,
    vol: pulse::volume::ChannelVolumes,
    context: &Rc<RefCell<Context>>,
) {
    let mut introspector = context.borrow_mut().introspect();
    match ident.entry_type {
        EntryType::Sink => {
            introspector.set_sink_volume_by_index(ident.index, &vol, None);
        }
        EntryType::SinkInput => {
            introspector.set_sink_input_volume(ident.index, &vol, None);
        }
        EntryType::Source => {
            introspector.set_source_volume_by_index(ident.index, &vol, None);
        }
        EntryType::SourceOutput => {
            introspector.set_source_output_volume(ident.index, &vol, None);
        }
    };
}

fn move_entry_to_parent(
    ident: EntryIdentifier,
    parent: EntryIdentifier,
    context: &Rc<RefCell<Context>>,
) {
    let mut introspector = context.borrow_mut().introspect();

    match ident.entry_type {
        EntryType::SinkInput => {
            introspector.move_sink_input_by_index(
                ident.index,
                parent.index,
                Some(Box::new(move |_| {
                    (*INFO_SX).get().send(parent).unwrap();
                    (*INFO_SX).get().send(ident).unwrap();
                })),
            );
        }
        EntryType::SourceOutput => {
            introspector.move_source_output_by_index(
                ident.index,
                parent.index,
                Some(Box::new(move |_| {
                    (*INFO_SX).get().send(parent).unwrap();
                    (*INFO_SX).get().send(ident).unwrap();
                })),
            );
        }
        _ => {}
    };
}

fn set_suspend(ident: EntryIdentifier, suspend: bool, context: &Rc<RefCell<Context>>) {
    let mut introspector = context.borrow_mut().introspect();
    match ident.entry_type {
        EntryType::Sink => {
            introspector.suspend_sink_by_index(ident.index, suspend, None);
        }
        EntryType::Source => {
            introspector.suspend_source_by_index(ident.index, suspend, None);
        }
        _ => {}
    };
}

fn kill_entry(ident: EntryIdentifier, context: &Rc<RefCell<Context>>) {
    let mut introspector = context.borrow_mut().introspect();
    match ident.entry_type {
        EntryType::SinkInput => {
            introspector.kill_sink_input(ident.index, |_| {});
        }
        EntryType::SourceOutput => {
            introspector.kill_source_output(ident.index, |_| {});
        }
        _ => {}
    };
}

fn set_mute(ident: EntryIdentifier, mute: bool, context: &Rc<RefCell<Context>>) {
    let mut introspector = context.borrow_mut().introspect();
    match ident.entry_type {
        EntryType::Sink => {
            introspector.set_sink_mute_by_index(ident.index, mute, None);
        }
        EntryType::SinkInput => {
            introspector.set_sink_input_mute(ident.index, mute, None);
        }
        EntryType::Source => {
            introspector.set_source_mute_by_index(ident.index, mute, None);
        }
        EntryType::SourceOutput => {
            introspector.set_source_output_mute(ident.index, mute, None);
        }
    };
}

pub fn remove_failed_monitors(index: &u32,
                                  x: &mut (
        Rc<RefCell<Stream>>,
        Option<u32>,
        cb_channel::Sender<u32>,
    )) -> bool {
    match x.0.borrow_mut().get_state() {
        pulse::stream::State::Failed => {
            info!(
                "[PAInterface] Disconnecting {} sink input monitor (failed state)",
                index
            );
            false
        }
        pulse::stream::State::Terminated => {
            info!(
                "[PAInterface] Disconnecting {} sink input monitor (failed state)",
                index
            );
            false
        }
        _ => true,
    }
}

pub fn create_monitors(
    mainloop: &Rc<RefCell<Mainloop>>,
    context: &Rc<RefCell<Context>>,
    sink_monitors: &mut Monitors,
    sink_input_monitors: &mut Monitors,
    source_monitors: &mut Monitors,
    _source_output_monitors: &mut Monitors,
    targets: &Vec<(Entry, Option<u32>)>,
) {
    sink_monitors.retain(remove_failed_monitors);
    sink_input_monitors.retain(remove_failed_monitors);
    source_monitors.retain(remove_failed_monitors);

    sink_monitors.retain(|k, _| {
        targets.iter().find(|(e, _)| {
            e.entry_type == EntryType::Sink && e.index == *k
        }) != None
    });
    sink_input_monitors.retain(|k, _| {
        targets.iter().find(|(e, _)| {
            e.entry_type == EntryType::SinkInput && e.index == *k
        }) != None
    });
    source_monitors.retain(|k, _| {
        targets.iter().find(|(e, _)| {
            e.entry_type == EntryType::Source && e.index == *k
        }) != None
    });
}

pub fn create_monitor_for_entry(
    mainloop: &Rc<RefCell<Mainloop>>,
    context: &Rc<RefCell<Context>>,
    sink_monitors: &mut Monitors,
    sink_input_monitors: &mut Monitors,
    source_monitors: &mut Monitors,
    _source_output_monitors: &mut Monitors,
    entry: Entry,
    monitor_src: Option<u32>,
) {
    log::error!("CREATE MON FOR {:?} {}", entry.entry_type, entry.index);
    let (sx, rx) = cb_channel::unbounded();
    let mut source_index = None;
    let mut stream_index = None;
    let mut monitors = None;
    match entry.entry_type {
        EntryType::SinkInput => {
            monitors = Some(sink_input_monitors);
            source_index = monitor_src;
            stream_index = Some(entry.index);
        }
        EntryType::Sink => {
            monitors = Some(sink_monitors);
            source_index = entry.monitor_source;
            stream_index = None;
        }
        EntryType::SourceOutput => {}
        EntryType::Source => {
            monitors = Some(source_monitors);
            source_index = Some(entry.index);
            stream_index = None;
        }
    };
    if let Some(ms) = monitors {
        if ms.contains_key(&entry.index) {
            return;
        }
        ms.insert(
            entry.index,
            (
                monitor::create(
                    &mainloop,
                    &context,
                    &*SPEC,
                    entry.entry_type,
                    source_index,
                    stream_index,
                    rx,
                )
                .unwrap(),
                monitor_src,
                sx,
            ),
        );
    }
}
