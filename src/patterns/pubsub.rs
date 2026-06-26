use crate::transport::traits::Connection;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

/// A network peer behind a shared, lockable connection handle.
pub type SharedPeer = Arc<Mutex<Box<dyn Connection>>>;

pub struct PubSubManager {
    sender: broadcast::Sender<(String, Bytes)>,
    network_peers: Arc<Mutex<Vec<SharedPeer>>>,
}

impl PubSubManager {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            network_peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_peer(&self, peer: SharedPeer) {
        self.network_peers.lock().await.push(peer);
    }

    pub async fn publish(&self, topic: String, message: Bytes) -> Result<()> {
        let _ = self.sender.send((topic.clone(), message.clone()));

        let mut peers = self.network_peers.lock().await;
        let mut to_remove = Vec::new();

        let mut payload = Vec::new();
        payload.push(topic.len() as u8);
        payload.extend_from_slice(topic.as_bytes());
        payload.extend_from_slice(&message);
        let final_msg = Bytes::from(payload);

        for (idx, peer_mutex) in peers.iter().enumerate() {
            let mut peer = peer_mutex.lock().await;
            if peer.send(final_msg.clone()).await.is_err() {
                to_remove.push(idx);
            }
        }

        for idx in to_remove.into_iter().rev() {
            peers.remove(idx);
        }

        Ok(())
    }

    pub fn subscribe(&self, _topic: &str) -> broadcast::Receiver<(String, Bytes)> {
        self.sender.subscribe()
    }
}
