use super::traits::{Connection, Listener, Transport};
use anyhow::Result;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl Connection for TcpConnection {
    async fn send(&mut self, data: Bytes) -> Result<()> {
        let len = (data.len() as u32).to_be_bytes();
        self.stream.write_all(&len).await?;
        self.stream.write_all(&data).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Bytes> {
        let mut len_buf = [0u8; 4];
        let n = self.stream.read_exact(&mut len_buf).await;
        if n.is_err() {
            return Err(anyhow::anyhow!("Connection closed or read error"));
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        // rudimentary safeguard against overly large allocations
        if len > 10 * 1024 * 1024 {
            return Err(anyhow::anyhow!("Payload too large"));
        }

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn close(&mut self) -> Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.stream.peer_addr().ok()
    }
}

pub struct TcpTransportListener {
    listener: TcpListener,
}

#[async_trait::async_trait]
impl Listener for TcpTransportListener {
    async fn accept(&mut self) -> Result<Box<dyn Connection>> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Box::new(TcpConnection::new(stream)))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}

pub struct TcpTransport;

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn bind(&self, addr: &str) -> Result<Box<dyn Listener>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Box::new(TcpTransportListener { listener }))
    }

    async fn connect(&self, addr: &str) -> Result<Box<dyn Connection>> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Box::new(TcpConnection::new(stream)))
    }
}
