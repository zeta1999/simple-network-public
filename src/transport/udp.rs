use super::traits::DatagramTransport;
use anyhow::Result;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct UdpDatagramTransport {
    socket: Arc<UdpSocket>,
}

impl UdpDatagramTransport {
    pub async fn bind(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
        })
    }
}

#[async_trait::async_trait]
impl DatagramTransport for UdpDatagramTransport {
    async fn send_to(&self, data: Bytes, target: &str) -> Result<()> {
        self.socket.send_to(&data, target).await?;
        Ok(())
    }

    async fn recv_from(&self) -> Result<(Bytes, SocketAddr)> {
        let mut buf = vec![0u8; 65536];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        buf.truncate(len);
        Ok((Bytes::from(buf), addr))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}
