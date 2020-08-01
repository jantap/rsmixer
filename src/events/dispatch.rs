
use super::common::*;

pub struct Dispatch<T: Send + Message + 'static>(
    Arc<RwLock<Option<Sender<T>>>>,
    Storage<cb_channel::Sender<T>>,
);

impl<T: Send + Message + Clone + std::fmt::Debug + 'static> Dispatch<T> {
    pub fn new() -> Self {
        Self {
            0: Arc::new(RwLock::new(None)),
            1: Storage::new(),
        }
    }

    pub async fn register(&self, sender: Sender<T>, sync_sender: cb_channel::Sender<T>) {
        let mut bis = self.0.write().await;
        *bis = Some(sender);
        self.1.set(sync_sender);

        // unsafe {
        //     let s = self.1.;
        //     *s = Some(sync_sender);
        // };
    }

    pub async fn event(&self, ev: T) {
        match self.0.read().await.as_ref() {
            Some(s) => {
                match s.send(ev.clone()) {
                    _ => {}
                };
            }
            None => {}
        }
    }

    pub fn sync_event(&self, ev: T) {
        match self.1.try_get() {
            Some(s) => {
                match s.send(ev) {
                    Err(_) => {}
                    _ => {}
                };
            }
            None => {}
        }
    }
}
