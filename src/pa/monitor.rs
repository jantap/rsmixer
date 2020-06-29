use super::common::*;
use pulse::stream::PeekResult;
use std::convert::TryInto;

fn slice_to_4_bytes(slice: &[u8]) -> [u8; 4] {
    slice.try_into().expect("slice with incorrect length")
}

pub fn create(
    p_mainloop: &Rc<RefCell<Mainloop>>,
    p_context: &Rc<RefCell<Context>>,
    p_spec: &pulse::sample::Spec,
    entry_type: EntryType,
    source_index: Option<u32>,
    stream_index: Option<u32>,
    close_rx: cb_channel::Receiver<u32>,
) -> Result<Rc<RefCell<Stream>>, RSError> {
    info!("[PADataInterface] Attempting to create new monitor stream");

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
                                let index = match entry_type {
                                    EntryType::SinkInput => stream_index.unwrap(),
                                    EntryType::Sink => source_index.unwrap(),
                                    EntryType::SourceOutput => source_index.unwrap(),
                                    EntryType::Source => source_index.unwrap(),
                                };

                                DISPATCH.sync_event(Letter::PeakVolumeUpdate(EntryIdentifier::new(entry_type, index), peak));

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
