use crate::{pa_interface::PAError, Entry, Letter, Result, DISPATCH};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    convert::TryInto,
    rc::Rc,
    sync::{Arc, Mutex},
};

use pulse::{
    callbacks::ListResult,
    context::{
        introspect::{SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo},
        Context,
    },
    mainloop::{
        api::Mainloop as MainloopTrait, //Needs to be in scope
        threaded::Mainloop,
    },
    stream::{PeekResult, Stream},
    volume::ChannelVolumes,
};

use async_std::task;
use lazy_static::lazy_static;
use log::{debug, error, info};

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub enum EntryType {
    Sink,
    SinkInput,
    Source,
    SourceOutput,
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct EntryIdentifier {
    pub entry_type: EntryType,
    pub index: u32,
}

impl Eq for EntryIdentifier {}

impl std::cmp::PartialOrd for EntryIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for EntryIdentifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let num = |x| match x {
            EntryType::Sink => 1,
            EntryType::Source => 2,
            EntryType::SinkInput => 3,
            EntryType::SourceOutput => 4,
        };

        if self.entry_type == other.entry_type && self.index == other.index {
            return std::cmp::Ordering::Equal;
        }

        let a = num(self.entry_type);
        let b = num(other.entry_type);

        if a > b {
            return std::cmp::Ordering::Greater;
        } else if b > a {
            return std::cmp::Ordering::Less;
        }
        if self.index > other.index {
            return std::cmp::Ordering::Greater;
        }
        return std::cmp::Ordering::Less;
    }
}

impl EntryIdentifier {
    pub fn new(entry_type: EntryType, index: u32) -> Self {
        Self { entry_type, index }
    }

    pub fn into_entry(&self) -> Result<Entry> {
        match self.entry_type {
            EntryType::Sink => {
                let sinks = SINKS.lock().unwrap();
                return match sinks.get(&self.index) {
                    Some(sink) => Ok(sink.to_entry()),
                    None => PAError::NoEntryError.boxed(),
                };
            }
            EntryType::SinkInput => {
                let sink_inputs = SINK_INPUTS.lock().unwrap();
                return match sink_inputs.get(&self.index) {
                    Some(sink_input) => Ok(sink_input.to_entry()),
                    None => PAError::NoEntryError.boxed(),
                };
            }
            EntryType::Source => {
                let sources = SOURCES.lock().unwrap();
                return match sources.get(&self.index) {
                    Some(source) => Ok(source.to_entry()),
                    None => PAError::NoEntryError.boxed(),
                };
            }
            EntryType::SourceOutput => {
                let source_outputs = SOURCE_OUTPUTS.lock().unwrap();
                return match source_outputs.get(&self.index) {
                    Some(source_output) => Ok(source_output.to_entry()),
                    None => PAError::NoEntryError.boxed(),
                };
            }
        };
    }
}

pub struct SinkType {
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: ChannelVolumes,
    pub monitor_source: u32,
}

pub struct SinkInputType {
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: ChannelVolumes,
    pub sink: u32,
}

pub struct SourceType {
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: ChannelVolumes,
}

pub struct SourceOutputType {
    pub index: u32,
    pub name: String,
    pub peak: f32,
    pub mute: bool,
    pub volume: ChannelVolumes,
    pub monitor_source: u32,
}

pub trait ToEntry {
    fn to_entry(&self) -> Entry;
}

impl ToEntry for SinkInputType {
    fn to_entry(&self) -> Entry {
        Entry {
            index: self.index,
            entry_type: EntryType::SinkInput,
            name: self.name.clone(),
            peak: self.peak,
            mute: self.mute,
            volume: self.volume.clone(),
            parent: Some(self.sink),
        }
    }
}

impl ToEntry for SinkType {
    fn to_entry(&self) -> Entry {
        Entry {
            index: self.index,
            entry_type: EntryType::Sink,
            name: self.name.clone(),
            peak: self.peak,
            mute: self.mute,
            volume: self.volume.clone(),
            parent: None,
        }
    }
}

impl ToEntry for SourceOutputType {
    fn to_entry(&self) -> Entry {
        Entry {
            index: self.index,
            entry_type: EntryType::Sink,
            name: self.name.clone(),
            peak: self.peak,
            mute: self.mute,
            volume: self.volume.clone(),
            parent: None,
        }
    }
}

impl ToEntry for SourceType {
    fn to_entry(&self) -> Entry {
        Entry {
            index: self.index,
            entry_type: EntryType::Sink,
            name: self.name.clone(),
            peak: self.peak,
            mute: self.mute,
            volume: self.volume.clone(),
            parent: None,
        }
    }
}

lazy_static! {
    pub static ref SINKS: Arc<Mutex<HashMap<u32, SinkType>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref SINK_INPUTS: Arc<Mutex<HashMap<u32, SinkInputType>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref SOURCES: Arc<Mutex<HashMap<u32, SourceType>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref SOURCE_OUTPUTS: Arc<Mutex<HashMap<u32, SourceOutputType>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref INFO_QUEUE: Arc<Mutex<HashSet<EntryIdentifier>>> =
        Arc::new(Mutex::new(HashSet::new()));
    pub static ref UI_UPDATE_ENTRY_QUEUE: Arc<Mutex<HashSet<EntryIdentifier>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

fn slice_to_4_bytes(slice: &[u8]) -> [u8; 4] {
    slice.try_into().expect("slice with incorrect length")
}

pub fn on_sink_info(res: ListResult<&SinkInfo>) {
    match res {
        ListResult::Item(i) => {
            debug!("[PADataInterface] Update {} sink info", i.index);
            let name = match &i.description {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            SINKS.lock().unwrap().insert(
                i.index,
                SinkType {
                    index: i.index,
                    name,
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: i.monitor_source,
                },
            );
            UI_UPDATE_ENTRY_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::Sink, i.index));
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
            SINK_INPUTS.lock().unwrap().insert(
                i.index,
                SinkInputType {
                    index: i.index,
                    name: n,
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    sink: i.sink,
                },
            );
            UI_UPDATE_ENTRY_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::SinkInput, i.index));
            INFO_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::Sink, i.sink));
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
            SOURCES.lock().unwrap().insert(
                i.index,
                SourceType {
                    index: i.index,
                    name,
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                },
            );
            UI_UPDATE_ENTRY_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::Source, i.index));
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
            SOURCE_OUTPUTS.lock().unwrap().insert(
                i.index,
                SourceOutputType {
                    index: i.index,
                    name: n,
                    peak: 0.0,
                    mute: i.mute,
                    volume: i.volume,
                    monitor_source: i.source,
                },
            );
            INFO_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::Source, i.source));
            UI_UPDATE_ENTRY_QUEUE
                .lock()
                .unwrap()
                .insert(EntryIdentifier::new(EntryType::SourceOutput, i.index));
        }
        _ => {}
    };
}

pub fn create_monitor(
    p_mainloop: &Rc<RefCell<Mainloop>>,
    p_context: &Rc<RefCell<Context>>,
    p_spec: &pulse::sample::Spec,
    entry_type: EntryType,
    source_index: Option<u32>,
    stream_index: Option<u32>,
) -> Result<Rc<RefCell<Stream>>> {
    info!("[PADataInterface] Attempting to create new monitor stream");

    let stream = Rc::new(RefCell::new(
        match Stream::new(&mut p_context.borrow_mut(), "RsMixer monitor", p_spec, None) {
            Some(stream) => stream,
            None => {
                return PAError::StreamCreateError.boxed();
            }
        },
    ));

    // Stream state change callback
    {
        debug!("[PADataInterface] Registering stream state change callback");
        let ml_ref = Rc::clone(&p_mainloop);
        let stream_ref = Rc::clone(&stream);
        stream
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*stream_ref.as_ptr()).get_state() };
                match state {
                    pulse::stream::State::Ready
                    | pulse::stream::State::Failed
                    | pulse::stream::State::Terminated => {
                        unsafe { (*ml_ref.as_ptr()).signal(false) };
                    }
                    _ => {}
                }
            })));
    }

    // for sink inputs we want to set monitor stream to sink
    if let Some(index) = stream_index {
        stream.borrow_mut().set_monitor_stream(index).unwrap();
    }

    let s = match source_index {
        Some(i) => i.to_string(),
        None => String::new(),
    };

    debug!("[PADataInterface] Connecting stream");
    match stream.borrow_mut().connect_record(
        if source_index.is_some() {
            Some(&s.as_str())
        } else {
            None
        },
        None,
        pulse::stream::flags::PEAK_DETECT | pulse::stream::flags::ADJUST_LATENCY,
    ) {
        Ok(_) => {}
        Err(_) => {
            return PAError::StreamCreateError.boxed();
        }
    };

    debug!("[PADataInterface] Waiting for stream to be ready");
    loop {
        match stream.borrow_mut().get_state() {
            pulse::stream::State::Ready => {
                break;
            }
            pulse::stream::State::Failed | pulse::stream::State::Terminated => {
                error!("[PADataInterface] Stream state failed/terminated");
                return PAError::StreamCreateError.boxed();
            }
            _ => {
                p_mainloop.borrow_mut().wait();
            }
        }
    }
    stream.borrow_mut().set_state_callback(None);

    {
        info!("[PADataInterface] Registering stream read callback");
        let ml_ref = Rc::clone(&p_mainloop);
        let stream_ref = Rc::clone(&stream);
        stream.borrow_mut().set_read_callback(Some(Box::new(move |_size: usize| {
            let remove_failed = || {
                error!("[PADataInterface] Monitor failed or terminated");
                match entry_type {
                    EntryType::SinkInput => {
                        SINK_INPUTS.lock().unwrap().remove(&stream_index.unwrap());
                    },
                    EntryType::Sink => {
                        SINKS.lock().unwrap().remove(&source_index.unwrap());
                    },
                    EntryType::SourceOutput => {
                        SOURCE_OUTPUTS.lock().unwrap().remove(&source_index.unwrap());
                    },
                    EntryType::Source => {
                        SOURCES.lock().unwrap().remove(&source_index.unwrap());
                    },
                };
            };
            let disconnect_stream = || {
                error!("[PADataInterface] Monitor existed while the sink (input)/source (output) was already gone");
                unsafe {
                    (*stream_ref.as_ptr()).discard().unwrap();
                    (*stream_ref.as_ptr()).disconnect().unwrap();
                    (*ml_ref.as_ptr()).signal(false);
                };
            };

            match unsafe {(*stream_ref.as_ptr()).get_state() }{
                pulse::stream::State::Failed => {
                    remove_failed();
                },
                pulse::stream::State::Terminated => {
                    remove_failed();
                },
                pulse::stream::State::Ready => {
                    match unsafe{ (*stream_ref.as_ptr()).peek() } {
                        Ok(res) => match res {
                            PeekResult::Data(data) => {
                                let size = data.len();
                                let data_slice = slice_to_4_bytes(&data[(size-4) .. size]);
                                let peak = f32::from_ne_bytes(data_slice).abs();
                                let mut peak_changed = false;
                                let index: u32;

                                match entry_type {
                                    EntryType::SinkInput => {
                                        index = stream_index.unwrap();
                                        let mut sink_inputs = SINK_INPUTS.lock().unwrap();
                                        match sink_inputs.get_mut(&index) {
                                            Some(sink_input) => {
                                                if sink_input.peak != peak {
                                                    peak_changed = true;
                                                    sink_input.peak = peak;
                                                }
                                            },
                                            None => {
                                                disconnect_stream();
                                                return;
                                            },
                                        }
                                    },
                                    EntryType::Sink => {
                                        index = source_index.unwrap();
                                        let mut sinks = SINKS.lock().unwrap();
                                        match sinks.get_mut(&index) {
                                            Some(sink) => {
                                                if sink.peak != peak {
                                                    peak_changed = true;
                                                    sink.peak = peak;
                                                }
                                            },
                                            None => {
                                                disconnect_stream();
                                                return;
                                            },
                                        }
                                    },
                                    EntryType::SourceOutput => {
                                        index = source_index.unwrap();
                                        let mut source_outputs = SOURCE_OUTPUTS.lock().unwrap();
                                        match source_outputs.get_mut(&index) {
                                            Some(source_output) => {
                                                if source_output.peak != peak {
                                                    peak_changed = true;
                                                    source_output.peak = peak;
                                                }
                                            },
                                            None => {
                                                disconnect_stream();
                                                return;
                                            },
                                        }
                                    },
                                    EntryType::Source => {
                                        index = source_index.unwrap();
                                        let mut sources = SOURCES.lock().unwrap();
                                        match sources.get_mut(&index) {
                                            Some(sources) => {
                                                if sources.peak != peak {
                                                    peak_changed = true;
                                                    sources.peak = peak;
                                                }
                                            },
                                            None => {
                                                disconnect_stream();
                                                return;
                                            },
                                        }
                                    },
                                };

                                if peak_changed {
                                    task::spawn(async move {
                                        DISPATCH.event(Letter::PeakVolumeUpdate(EntryIdentifier::new(entry_type, index), peak)).await;
                                    });
                                }

                                unsafe { (*stream_ref.as_ptr()).discard().unwrap(); };
                            },
                            PeekResult::Hole(_) => {
                                unsafe { (*stream_ref.as_ptr()).discard().unwrap(); };
                            },
                            _ => {},
                        },
                        Err(_) => {
                            remove_failed();
                        },
                    }
                },
                _ => {},
            };
            // unsafe {(*ml_ref.as_ptr()).signal(false)};
        })));
    }

    return Ok(stream);
}
