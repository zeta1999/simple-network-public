use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;

#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&mut self, data: Bytes) -> Result<()>;
    async fn recv(&mut self) -> Result<Bytes>;
    async fn close(&mut self) -> Result<()>;
    fn remote_addr(&self) -> Option<SocketAddr>;
}

#[async_trait]
pub trait Listener: Send + Sync {
    async fn accept(&mut self) -> Result<Box<dyn Connection>>;
    fn local_addr(&self) -> Result<SocketAddr>;
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn bind(&self, addr: &str) -> Result<Box<dyn Listener>>;
    async fn connect(&self, addr: &str) -> Result<Box<dyn Connection>>;
}

#[async_trait]
pub trait DatagramTransport: Send + Sync {
    async fn send_to(&self, data: Bytes, target: &str) -> Result<()>;
    async fn recv_from(&self) -> Result<(Bytes, SocketAddr)>;
    fn local_addr(&self) -> Result<SocketAddr>;
}
