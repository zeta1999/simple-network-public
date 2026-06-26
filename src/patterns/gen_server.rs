use crate::transport::traits::{Connection, Listener};
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait GenServerHandler: Send + Sync {
    async fn handle_call(&self, request: Bytes) -> Result<Bytes>;
    async fn handle_cast(&self, message: Bytes) -> Result<()>;
}

pub struct GenServer;

impl GenServer {
    pub async fn start_link(
        mut listener: Box<dyn Listener>,
        handler: Arc<dyn GenServerHandler>,
    ) -> Result<()> {
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok(mut conn) => {
                        let handler_clone = handler.clone();
                        tokio::spawn(async move {
                            // Loop until the connection closes (recv returns Err).
                            while let Ok(data) = conn.recv().await {
                                if data.is_empty() {
                                    break;
                                }
                                let msg_type = data[0];
                                let payload = data.slice(1..);

                                if msg_type == 0 {
                                    // Call
                                    match handler_clone.handle_call(payload).await {
                                        Ok(resp) => {
                                            if let Err(e) = conn.send(resp).await {
                                                eprintln!("Failed to send response: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Handler failed: {}", e);
                                            break;
                                        }
                                    }
                                } else {
                                    // Cast
                                    if let Err(e) = handler_clone.handle_cast(payload).await {
                                        eprintln!("Handler cast failed: {}", e);
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

pub struct GenServerClient {
    conn: Arc<Mutex<Box<dyn Connection>>>,
}

impl GenServerClient {
    pub fn new(conn: Box<dyn Connection>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn from_arc(conn: Arc<Mutex<Box<dyn Connection>>>) -> Self {
        Self { conn }
    }

    pub async fn call(&self, request: Bytes) -> Result<Bytes> {
        let mut framed = BytesMut::with_capacity(request.len() + 1);
        framed.put_u8(0); // 0 = Call
        framed.extend_from_slice(&request);

        let mut conn = self.conn.lock().await;
        conn.send(framed.freeze()).await?;
        let resp = conn.recv().await?;
        Ok(resp)
    }

    pub async fn cast(&self, message: Bytes) -> Result<()> {
        let mut framed = BytesMut::with_capacity(message.len() + 1);
        framed.put_u8(1); // 1 = Cast
        framed.extend_from_slice(&message);

        let mut conn = self.conn.lock().await;
        conn.send(framed.freeze()).await?;
        Ok(())
    }
}
