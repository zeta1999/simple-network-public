//! Tor onion carrier (feature `tor`).
//!
//! A drop-in [`Connection`] backed by an arti [`DataStream`], so the PQC
//! `SecureConnection` (or anything built on the transport seam) can run over
//! Tor with no protocol change — the "carrier swap" from NETWORKING.md.
//!
//! - [`TorTransport::connect`] dials a `host:port` or `.onion` address as a Tor
//!   client (NAT-free outbound).
//! - [`TorTransport::launch_onion`] hosts an onion service so a home machine is
//!   reachable without port-forwarding or a VPN, returning the generated
//!   `.onion` address and a [`Listener`] of inbound connections.
//!
//! Heavy dependency tree; exercising it needs a live Tor network, so this is
//! off by default and not run in the lean CI gate.

use super::traits::{Connection, Listener, Transport};
use anyhow::{anyhow, Result};
use arti_client::{DataStream, TorClient, TorClientConfig};
use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use safelog::DisplayRedacted;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tor_cell::relaycell::msg::Connected;
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_hsservice::{handle_rend_requests, HsNickname, RunningOnionService, StreamRequest};
use tor_proto::client::stream::IncomingStreamRequest;
use tor_rtcompat::PreferredRuntime;

const MAX_FRAME: usize = 10 * 1024 * 1024;

/// A [`Connection`] over a Tor [`DataStream`]. Length-prefixed framing,
/// identical on the wire to [`super::tcp::TcpConnection`].
pub struct TorConnection {
    stream: DataStream,
}

impl TorConnection {
    pub fn new(stream: DataStream) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl Connection for TorConnection {
    async fn send(&mut self, data: Bytes) -> Result<()> {
        let len = (data.len() as u32).to_be_bytes();
        self.stream.write_all(&len).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Bytes> {
        let mut len_buf = [0u8; 4];
        if self.stream.read_exact(&mut len_buf).await.is_err() {
            return Err(anyhow!("Connection closed or read error"));
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        if len > MAX_FRAME {
            return Err(anyhow!("Payload too large"));
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
        // Tor circuits intentionally have no observable peer socket address.
        None
    }
}

/// A bootstrapped Tor client usable as a [`Transport`].
pub struct TorTransport {
    client: Arc<TorClient<PreferredRuntime>>,
}

impl TorTransport {
    /// Bootstrap a Tor client (connects to the network; can take seconds).
    pub async fn bootstrapped() -> Result<Self> {
        let config = TorClientConfig::default();
        let client = TorClient::create_bootstrapped(config).await?;
        Ok(Self { client })
    }

    /// Reuse an already-bootstrapped client.
    pub fn from_client(client: Arc<TorClient<PreferredRuntime>>) -> Self {
        Self { client }
    }

    /// Host an onion service and accept inbound connections over it.
    ///
    /// Returns the generated `.onion` address (give it to clients) and a
    /// [`Listener`] yielding a [`TorConnection`] per inbound stream.
    pub async fn launch_onion(&self, nickname: &str) -> Result<(String, Box<dyn Listener>)> {
        let nick: HsNickname = nickname
            .parse()
            .map_err(|e| anyhow!("invalid onion nickname {nickname:?}: {e}"))?;
        let svc_config = OnionServiceConfigBuilder::default()
            .nickname(nick)
            .build()
            .map_err(|e| anyhow!("onion service config: {e}"))?;

        let (service, rend_requests) = self
            .client
            .launch_onion_service(svc_config)?
            .ok_or_else(|| anyhow!("onion service failed to start"))?;
        let onion = service
            .onion_address()
            .ok_or_else(|| anyhow!("onion service has no address yet"))?
            .display_unredacted()
            .to_string();

        let streams = tokio::sync::Mutex::new(handle_rend_requests(rend_requests).boxed());
        Ok((
            onion,
            Box::new(TorOnionListener {
                _service: service,
                streams,
            }),
        ))
    }
}

#[async_trait::async_trait]
impl Transport for TorTransport {
    async fn connect(&self, addr: &str) -> Result<Box<dyn Connection>> {
        let stream = self.client.connect(addr).await?;
        Ok(Box::new(TorConnection::new(stream)))
    }

    async fn bind(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        Err(anyhow!(
            "Tor accepts inbound via onion services; use TorTransport::launch_onion"
        ))
    }
}

/// Inbound side of a hosted onion service.
pub struct TorOnionListener {
    // Keep the service alive for as long as we accept on it.
    _service: Arc<RunningOnionService>,
    // Mutex so the listener is Sync (the boxed stream is Send-only); accept has
    // &mut self, so this never actually contends.
    streams: tokio::sync::Mutex<BoxStream<'static, StreamRequest>>,
}

#[async_trait::async_trait]
impl Listener for TorOnionListener {
    async fn accept(&mut self) -> Result<Box<dyn Connection>> {
        let streams = self.streams.get_mut();
        loop {
            let req = streams
                .next()
                .await
                .ok_or_else(|| anyhow!("onion service request stream ended"))?;
            match req.request() {
                IncomingStreamRequest::Begin(_) => {
                    let stream = req.accept(Connected::new_empty()).await?;
                    return Ok(Box::new(TorConnection::new(stream)));
                }
                // Ignore non-data requests (e.g. resolve) and keep listening.
                _ => continue,
            }
        }
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Err(anyhow!(
            "onion services have no socket address; use the .onion address from launch_onion"
        ))
    }
}
