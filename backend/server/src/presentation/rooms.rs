use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

const CHANNEL_CAP: usize = 512;

#[derive(Clone)]
pub struct RoomRegistry {
    inner: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
}

impl RoomRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, board_id: Uuid) -> broadcast::Receiver<String> {
        let mut map = self.inner.write().await;
        let tx = map.entry(board_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(CHANNEL_CAP);
            tx
        });
        tx.subscribe()
    }

    pub async fn publish(&self, board_id: Uuid, msg: String) {
        let map = self.inner.read().await;
        if let Some(tx) = map.get(&board_id) {
            let _ = tx.send(msg);
        }
    }
}
