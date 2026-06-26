use super::traits::{Connection, Listener, Transport};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{pki_types::ServerName, ClientConfig, ServerConfig};
use tokio_rustls::{
    client::TlsStream as ClientTlsStream, server::TlsStream as ServerTlsStream, TlsAcceptor,
    TlsConnector,
};

pub enum StreamMode {
    Client(ClientTlsStream<TcpStream>),
    Server(ServerTlsStream<TcpStream>),
}

pub struct TlsConnection {
    stream: StreamMode,
    remote_addr: Option<SocketAddr>,
}

impl TlsConnection {
    pub fn new_client(stream: ClientTlsStream<TcpStream>, remote_addr: Option<SocketAddr>) -> Self {
        Self {
            stream: StreamMode::Client(stream),
            remote_addr,
        }
    }

    pub fn new_server(stream: ServerTlsStream<TcpStream>, remote_addr: Option<SocketAddr>) -> Self {
        Self {
            stream: StreamMode::Server(stream),
            remote_addr,
        }
    }
}

#[async_trait::async_trait]
impl Connection for TlsConnection {
    async fn send(&mut self, data: Bytes) -> Result<()> {
        let len = (data.len() as u32).to_be_bytes();
        match &mut self.stream {
            StreamMode::Client(s) => {
                s.write_all(&len).await?;
                s.write_all(&data).await?;
            }
            StreamMode::Server(s) => {
                s.write_all(&len).await?;
                s.write_all(&data).await?;
            }
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Bytes> {
        let mut len_buf = [0u8; 4];
        let res = match &mut self.stream {
            StreamMode::Client(s) => s.read_exact(&mut len_buf).await,
            StreamMode::Server(s) => s.read_exact(&mut len_buf).await,
        };
        if res.is_err() {
            return Err(anyhow!("Connection closed or read error"));
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        if len > 10 * 1024 * 1024 {
            return Err(anyhow!("Payload too large"));
        }
        let mut buf = vec![0u8; len];
        match &mut self.stream {
            StreamMode::Client(s) => s.read_exact(&mut buf).await?,
            StreamMode::Server(s) => s.read_exact(&mut buf).await?,
        };
        Ok(Bytes::from(buf))
    }

    async fn close(&mut self) -> Result<()> {
        match &mut self.stream {
            StreamMode::Client(s) => s.shutdown().await?,
            StreamMode::Server(s) => s.shutdown().await?,
        }
        Ok(())
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
}

pub struct TlsTransportListener {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

#[async_trait::async_trait]
impl Listener for TlsTransportListener {
    async fn accept(&mut self) -> Result<Box<dyn Connection>> {
        let (stream, peer_addr) = self.listener.accept().await?;
        let tls_stream = self.acceptor.accept(stream).await?;
        Ok(Box::new(TlsConnection::new_server(
            tls_stream,
            Some(peer_addr),
        )))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}

pub struct TlsTransport {
    client_config: Arc<ClientConfig>,
    server_config: Arc<ServerConfig>,
    domain: String,
}

impl TlsTransport {
    pub fn new(
        client_config: Arc<ClientConfig>,
        server_config: Arc<ServerConfig>,
        domain: String,
    ) -> Self {
        Self {
            client_config,
            server_config,
            domain,
        }
    }
}

#[async_trait::async_trait]
impl Transport for TlsTransport {
    async fn bind(&self, addr: &str) -> Result<Box<dyn Listener>> {
        let listener = TcpListener::bind(addr).await?;
        let acceptor = TlsAcceptor::from(self.server_config.clone());
        Ok(Box::new(TlsTransportListener { listener, acceptor }))
    }

    async fn connect(&self, addr: &str) -> Result<Box<dyn Connection>> {
        let stream = TcpStream::connect(addr).await?;
        let peer_addr = stream.peer_addr().ok();
        let connector = TlsConnector::from(self.client_config.clone());
        let domain = ServerName::try_from(self.domain.clone())
            .map_err(|_| anyhow!("Invalid domain name"))?
            .to_owned();
        let tls_stream = connector.connect(domain, stream).await?;
        Ok(Box::new(TlsConnection::new_client(tls_stream, peer_addr)))
    }
}
