use super::common::*;
use pulse::stream::PeekResult;
use std::collections::HashSet;
use std::convert::TryInto;

pub struct Monitor {
    stream: Rc<RefCell<Stream>>,
    monitor_src: Option<u32>,
    exit_sender: cb_channel::Sender<u32>,
}

pub struct Monitors(HashMap<EntryIdentifier, Monitor>);

impl Default for Monitors {
    fn default() -> Self {
        Self { 0: HashMap::new() }
    }
}

impl Monitors {
    pub fn insert(&mut self, ident: EntryIdentifier, monitor: Monitor) -> Option<Monitor> {
        self.0.insert(ident, monitor)
    }

    pub fn filter(
        &mut self,
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        targets: &HashMap<EntryIdentifier, Option<u32>>,
    ) {
        // remove failed streams
        // then send exit signal if stream is unwanted
        self.0.retain(|ident, monitor| {
            match monitor.stream.borrow_mut().get_state() {
                pulse::stream::State::Terminated | pulse::stream::State::Failed => {
                    info!(
                        "[PAInterface] Disconnecting {} sink input monitor (failed state)",
                        ident.index
                    );
                    return false;
                }
                _ => {}
            };

            if targets.get(ident) == None {
                match monitor.exit_sender.send(0) {
                    _ => {}
                }
            }

            true
        });

        targets
            .iter()
            .for_each(|(ident, monitor_src)| match self.0.get(ident) {
                None => {
                    self.create_monitor(mainloop, context, *ident, *monitor_src);
                }
                _ => {}
            });
    }

    fn create_monitor(
        &mut self,
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        ident: EntryIdentifier,
        monitor_src: Option<u32>,
    ) {
        if self.0.contains_key(&ident) {
            return;
        }
        let (sx, rx) = cb_channel::unbounded();
        if let Ok(stream) = create(&mainloop, &context, &*SPEC, ident, monitor_src, rx) {
            self.0.insert(
                ident,
                Monitor {
                    stream,
                    monitor_src,
                    exit_sender: sx,
                },
            );
        }
    }
}

fn slice_to_4_bytes(slice: &[u8]) -> [u8; 4] {
    slice.try_into().expect("slice with incorrect length")
}

fn create(
    p_mainloop: &Rc<RefCell<Mainloop>>,
    p_context: &Rc<RefCell<Context>>,
    p_spec: &pulse::sample::Spec,
    ident: EntryIdentifier,
    source_index: Option<u32>,
    close_rx: cb_channel::Receiver<u32>,
) -> Result<Rc<RefCell<Stream>>, RSError> {
    info!("[PADataInterface] Attempting to create new monitor stream");

    let stream_index = if ident.entry_type == EntryType::SinkInput {
        Some(ident.index)
    } else {
        None
    };

    let stream = Rc::new(RefCell::new(
        match Stream::new(&mut p_context.borrow_mut(), "RsMixer monitor", p_spec, None) {
            Some(stream) => stream,
            None => {
                return Err(RSError::StreamCreateError);
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

    let x;
    let mut s = None;
    if let Some(i) = source_index {
        x = i.to_string();
        s = Some(x.as_str());
    }

    debug!("[PADataInterface] Connecting stream");
    match stream.borrow_mut().connect_record(
        s,
        Some(&pulse::def::BufferAttr {
            maxlength: std::u32::MAX,
            tlength: std::u32::MAX,
            prebuf: std::u32::MAX,
            minreq: 0,
            fragsize: 4,
        }),
        pulse::stream::flags::PEAK_DETECT | pulse::stream::flags::ADJUST_LATENCY,
    ) {
        Ok(_) => {}
        Err(_) => {
            return Err(RSError::StreamCreateError);
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
                return Err(RSError::StreamCreateError);
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
            };
            let disconnect_stream = || {
                error!("[PADataInterface] Monitor existed while the sink (input)/source (output) was already gone");
                unsafe {
                    (*stream_ref.as_ptr()).disconnect().unwrap();
                    (*ml_ref.as_ptr()).signal(false);
                };
            };

            if let Ok(_) = close_rx.try_recv() {
                disconnect_stream();
                return;
            }

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

                                DISPATCH.sync_event(Letter::PeakVolumeUpdate(ident, peak));

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
            // unsafe {(*ml_ref.get_mut().unwrap()).signal(false)};
        })));
    }

    return Ok(stream);
}
