use std::convert::TryInto;

use pulse::stream::PeekResult;

use super::{common::*, sync_loop::ACTIONS_SX};

pub struct Monitor {
    stream: Rc<RefCell<Stream>>,
    exit_sender: cb_channel::Sender<u32>,
}

pub struct Monitors {
    monitors: HashMap<EntryIdentifier, Monitor>,
    errors: HashMap<EntryIdentifier, usize>,
}

impl Default for Monitors {
    fn default() -> Self {
        Self {
            monitors: HashMap::new(),
            errors: HashMap::new(),
        }
    }
}

impl Monitors {
    pub fn filter(
        &mut self,
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        targets: &HashMap<EntryIdentifier, Option<u32>>,
    ) {
        // remove failed streams
        // then send exit signal if stream is unwanted
        self.monitors.retain(|ident, monitor| {
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
                let _ = monitor.exit_sender.send(0);
            }

            true
        });

        targets.iter().for_each(|(ident, monitor_src)| {
            if self.monitors.get(ident).is_none() {
                self.create_monitor(mainloop, context, *ident, *monitor_src);
            }
        });
    }

    fn create_monitor(
        &mut self,
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        ident: EntryIdentifier,
        monitor_src: Option<u32>,
    ) {
        if let Some(count) = self.errors.get(&ident) {
            if *count >= 5 {
                self.errors.remove(&ident);
                (*ACTIONS_SX)
                    .get()
                    .send(EntryUpdate::EntryRemoved(ident))
                    .unwrap();
            }
        }
        if self.monitors.contains_key(&ident) {
            return;
        }
        let (sx, rx) = cb_channel::unbounded();
        if let Ok(stream) = create(&mainloop, &context, &*SPEC, ident, monitor_src, rx) {
            self.monitors.insert(
                ident,
                Monitor {
                    stream,
                    exit_sender: sx,
                },
            );
            self.errors.remove(&ident);
        } else {
            self.error(&ident);
        }
    }

    fn error(&mut self, ident: &EntryIdentifier) {
        let count = match self.errors.get(&ident) {
            Some(x) => *x,
            None => 0,
        };
        self.errors.insert(*ident, count + 1);
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
) -> Result<Rc<RefCell<Stream>>, RsError> {
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
                return Err(RsError::StreamCreateError);
            }
        },
    ));

    // Stream state change callback
    {
        debug!("[PADataInterface] Registering stream state change callback");
        let ml_ref = Rc::clone(&p_mainloop);
        let stream_ref = Rc::downgrade(&stream);
        stream
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*(*stream_ref.as_ptr()).as_ptr()).get_state() };
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
        pulse::stream::FlagSet::PEAK_DETECT | pulse::stream::FlagSet::ADJUST_LATENCY,
    ) {
        Ok(_) => {}
        Err(_) => {
            return Err(RsError::StreamCreateError);
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
                return Err(RsError::StreamCreateError);
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
        let stream_ref = Rc::downgrade(&stream);
        stream.borrow_mut().set_read_callback(Some(Box::new(move |_size: usize| {
            let remove_failed = || {
                error!("[PADataInterface] Monitor failed or terminated");
            };
            let disconnect_stream = || {
                warn!("[PADataInterface] {:?} Monitor existed while the sink (input)/source (output) was already gone", ident);
                unsafe {
                    (*(*stream_ref.as_ptr()).as_ptr()).disconnect().unwrap();
                    (*ml_ref.as_ptr()).signal(false);
                };
            };

            if close_rx.try_recv().is_ok() {
                disconnect_stream();
                return;
            }

            match unsafe {(*(*stream_ref.as_ptr()).as_ptr()).get_state() }{
                pulse::stream::State::Failed => {
                    remove_failed();
                },
                pulse::stream::State::Terminated => {
                    remove_failed();
                },
                pulse::stream::State::Ready => {
                    match unsafe{ (*(*stream_ref.as_ptr()).as_ptr()).peek() } {
                        Ok(res) => match res {
                            PeekResult::Data(data) => {
                                let size = data.len();
                                let data_slice = slice_to_4_bytes(&data[(size-4) .. size]);
                                let peak = f32::from_ne_bytes(data_slice).abs();

                                if (*ACTIONS_SX).get().send(EntryUpdate::PeakVolumeUpdate(ident, peak)).is_err() {
                                    disconnect_stream();
                                }

                                unsafe { (*(*stream_ref.as_ptr()).as_ptr()).discard().unwrap(); };
                            },
                            PeekResult::Hole(_) => {
                                unsafe { (*(*stream_ref.as_ptr()).as_ptr()).discard().unwrap(); };
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

    Ok(stream)
}
