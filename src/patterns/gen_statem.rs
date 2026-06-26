use crate::transport::traits::{Connection, Listener};
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait GenStatemHandler: Send + Sync {
    async fn handle_event(&mut self, event: Bytes) -> Result<Bytes>;
}

pub struct GenStatem;

impl GenStatem {
    pub async fn start_link(
        mut listener: Box<dyn Listener>,
        handler: Arc<Mutex<dyn GenStatemHandler>>,
    ) -> Result<()> {
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok(mut conn) => {
                        let handler_clone = handler.clone();
                        tokio::spawn(async move {
                            // Loop until the connection closes (recv returns Err).
                            while let Ok(data) = conn.recv().await {
                                let mut h = handler_clone.lock().await;
                                match h.handle_event(data).await {
                                    Ok(resp) => {
                                        if let Err(e) = conn.send(resp).await {
                                            eprintln!("Failed to send gen_statem resp: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("GenStatem handler failed: {}", e);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => eprintln!("Listener accept error: {}", e),
                }
            }
        });
        Ok(())
    }
}

pub struct GenStatemClient {
    conn: Arc<Mutex<Box<dyn Connection>>>,
}

impl GenStatemClient {
    pub fn new(conn: Box<dyn Connection>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn from_arc(conn: Arc<Mutex<Box<dyn Connection>>>) -> Self {
        Self { conn }
    }

    pub async fn send_event(&self, event: Bytes) -> Result<Bytes> {
        let mut conn = self.conn.lock().await;
        conn.send(event).await?;
        let resp = conn.recv().await?;
        Ok(resp)
    }
}
