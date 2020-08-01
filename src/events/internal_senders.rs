use super::common::*;

#[derive(Clone)]
pub struct Senders<T: Send + Message + 'static>(pub Arc<RwLock<HashMap<u32, Sender<T>>>>);

impl<T: Send + Message + 'static> Senders<T> {
    pub fn new() -> Self {
        Self {
            0: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, i: u32, sender: Sender<T>) {
        self.0.write().await.insert(i, sender);
    }
}
