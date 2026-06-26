use super::super::traits::Connection;
use anyhow::Result;
use bytes::Bytes;
use std::net::SocketAddr;

pub struct ThrottledConnection<C: Connection> {
    inner: C,
    #[allow(dead_code)]
    bytes_per_second: usize,
}

impl<C: Connection> ThrottledConnection<C> {
    pub fn new(inner: C, bytes_per_second: usize) -> Self {
        Self {
            inner,
            bytes_per_second,
        }
    }
}

#[async_trait::async_trait]
impl<C: Connection + Send + Sync> Connection for ThrottledConnection<C> {
    async fn send(&mut self, data: Bytes) -> Result<()> {
        // TODO: Implement actual rate limiting token bucket
        self.inner.send(data).await
    }

    async fn recv(&mut self) -> Result<Bytes> {
        // TODO: Implement actual rate limiting token bucket
        self.inner.recv().await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.inner.remote_addr()
    }
}
