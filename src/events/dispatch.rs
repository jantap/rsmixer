use super::common::*;

pub struct Dispatch<T: Send + Message + 'static>(
    Arc<RwLock<Option<Sender<T>>>>,
    Storage<cb_channel::Sender<T>>,
);

impl<T: Send + Message + Clone + std::fmt::Debug + 'static> Default for Dispatch<T> {
    fn default() -> Self {
        Self {
            0: Arc::new(RwLock::new(None)),
            1: Storage::new(),
        }
    }
}

impl<T: Send + Message + Clone + std::fmt::Debug + 'static> Dispatch<T> {
    pub async fn register(&self, sender: Sender<T>, sync_sender: cb_channel::Sender<T>) {
        log::error!("register11");
        let mut bis = self.0.write().await;
        *bis = Some(sender);
        self.1.set(sync_sender);
        log::error!("register22");

        // unsafe {
        //     let s = self.1.;
        //     *s = Some(sync_sender);
        // };
    }

    pub async fn event(&self, ev: T) {
        if let Some(s) = self.0.read().await.as_ref() {
            let _ = s.send(ev.clone());
        }
    }

    pub fn sync_event(&self, ev: T) {
        if let Some(s) = self.1.try_get() {
            let _ = s.send(ev);
        }
    }
}
