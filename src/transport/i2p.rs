//! I2P carrier over the SAM v3 bridge (feature `i2p`).
//!
//! A drop-in [`Connection`] backed by an I2P stream, so the PQC
//! `SecureConnection` (or anything built on the transport seam) can run over
//! I2P with no protocol change — the same "carrier swap" as [`super::tor`], but
//! reaching the I2P network instead of Tor.
//!
//! Unlike Tor's arti there is no pure-Rust I2P router, so this speaks the SAM
//! v3 protocol to a *locally running* router (i2pd or Java I2P) with SAM
//! enabled — by default `127.0.0.1:7656`. We talk SAM directly over tokio
//! sockets, so the carrier pulls in no extra dependencies.
//!
//! - [`I2pTransport::create`] opens a SAM `STREAM` session (a fresh transient
//!   destination by default) and is reusable for many connections.
//! - [`Transport::connect`] dials a `*.b32.i2p` / full-destination peer
//!   (NAT-free outbound).
//! - [`Transport::bind`] returns a [`Listener`] that `STREAM ACCEPT`s inbound
//!   streams on the session, so a home machine is reachable without
//!   port-forwarding. Hand peers [`I2pTransport::my_destination`].
//!
//! Exercising it needs a live I2P router, so the feature is off by default and
//! not run in the lean CI gate. The SAM reply parsing is unit-tested without a
//! router.

use super::traits::{Connection, Listener, Transport};
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const MAX_FRAME: usize = 10 * 1024 * 1024;
/// Default SAM bridge address (i2pd / Java I2P listen here when SAM is on).
const DEFAULT_SAM: &str = "127.0.0.1:7656";
const SAM_MIN: &str = "3.0";
const SAM_MAX: &str = "3.3";

/// A buffered SAM socket. `BufReader` lets us read the line-oriented SAM
/// handshake without over-reading into the raw stream that follows, and it
/// delegates writes straight through to the inner socket.
type SamSocket = BufReader<TcpStream>;

/// A [`Connection`] over an I2P SAM stream. Length-prefixed framing, identical
/// on the wire to [`super::tcp::TcpConnection`] and [`super::tor::TorConnection`].
pub struct I2pConnection {
    stream: SamSocket,
}

impl I2pConnection {
    fn new(stream: SamSocket) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl Connection for I2pConnection {
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
        // I2P streams intentionally have no observable peer socket address.
        None
    }
}

/// A SAM `STREAM` session usable as a [`Transport`]. The session's control
/// socket is held open for the session's lifetime; dropping the transport tears
/// the session (and any [`I2pListener`] still accepting on it) down.
pub struct I2pTransport {
    sam_addr: String,
    session_id: String,
    /// Our own (public) destination, base64 — hand this to peers so they can
    /// `connect` to us. Resolved via `NAMING LOOKUP NAME=ME`.
    my_dest: String,
    /// Kept open for the lifetime of the session; the router drops the session
    /// when this socket closes.
    _control: SamSocket,
}

impl I2pTransport {
    /// Open a SAM `STREAM` session against the router. `sam_addr` defaults to
    /// `127.0.0.1:7656`; `nickname` disambiguates the session id on the router.
    /// Uses a fresh transient destination (a new identity every run).
    pub async fn create(sam_addr: Option<&str>, nickname: &str) -> Result<Self> {
        let sam_addr = sam_addr.unwrap_or(DEFAULT_SAM).to_string();
        let mut control = connect_sam(&sam_addr).await?;
        sam_hello(&mut control).await?;

        let session_id = format!("simple-network-{nickname}");
        write_line(
            &mut control,
            &format!("SESSION CREATE STYLE=STREAM ID={session_id} DESTINATION=TRANSIENT"),
        )
        .await?;
        sam_check_ok(&read_line(&mut control).await?)?;

        // Resolve our own public destination so callers can advertise it.
        write_line(&mut control, "NAMING LOOKUP NAME=ME").await?;
        let kv = sam_check_ok(&read_line(&mut control).await?)?;
        let my_dest = kv
            .get("VALUE")
            .cloned()
            .ok_or_else(|| anyhow!("SAM NAMING REPLY missing VALUE (our destination)"))?;

        Ok(Self {
            sam_addr,
            session_id,
            my_dest,
            _control: control,
        })
    }

    /// Our public destination (base64). Peers pass this (or its `.b32.i2p`
    /// short form) to [`Transport::connect`].
    pub fn my_destination(&self) -> &str {
        &self.my_dest
    }
}

#[async_trait::async_trait]
impl Transport for I2pTransport {
    async fn connect(&self, addr: &str) -> Result<Box<dyn Connection>> {
        // Callers of the generic Transport pass `host:port` (see TcpTransport).
        // For SAM, the destination and the stream port are distinct fields:
        // DESTINATION is the bare base64 dest / `.b32.i2p`, and any port goes in
        // TO_PORT (SAM 3.2+; negotiated up to MAX_VERSION).
        let (dest, port) = split_dest_port(addr);
        let mut sock = connect_sam(&self.sam_addr).await?;
        sam_hello(&mut sock).await?;
        let mut cmd = format!(
            "STREAM CONNECT ID={} DESTINATION={dest} SILENT=false",
            self.session_id
        );
        if let Some(port) = port {
            cmd.push_str(&format!(" TO_PORT={port}"));
        }
        write_line(&mut sock, &cmd).await?;
        sam_check_ok(&read_line(&mut sock).await?)?;
        // After STREAM STATUS RESULT=OK the socket carries the raw peer stream.
        Ok(Box::new(I2pConnection::new(sock)))
    }

    async fn bind(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        Ok(Box::new(I2pListener {
            sam_addr: self.sam_addr.clone(),
            session_id: self.session_id.clone(),
        }))
    }
}

/// Inbound side of a SAM session: each `accept` issues a fresh `STREAM ACCEPT`
/// on the session. Requires the owning [`I2pTransport`] to stay alive (it holds
/// the session's control socket).
pub struct I2pListener {
    sam_addr: String,
    session_id: String,
}

#[async_trait::async_trait]
impl Listener for I2pListener {
    async fn accept(&mut self) -> Result<Box<dyn Connection>> {
        let mut sock = connect_sam(&self.sam_addr).await?;
        sam_hello(&mut sock).await?;
        write_line(
            &mut sock,
            &format!("STREAM ACCEPT ID={} SILENT=false", self.session_id),
        )
        .await?;
        sam_check_ok(&read_line(&mut sock).await?)?;
        // With SILENT=false the router emits the remote destination on its own
        // line before the raw stream begins; consume it.
        let _remote_dest = read_line(&mut sock).await?;
        Ok(Box::new(I2pConnection::new(sock)))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Err(anyhow!(
            "I2P streams have no socket address; advertise I2pTransport::my_destination"
        ))
    }
}

// --------------------------------------------------------------------------
// SAM v3 protocol helpers
// --------------------------------------------------------------------------

async fn connect_sam(addr: &str) -> Result<SamSocket> {
    let stream = TcpStream::connect(addr).await.map_err(|e| {
        anyhow!("connect to SAM bridge {addr}: {e} (is the I2P router running with SAM enabled?)")
    })?;
    Ok(BufReader::new(stream))
}

async fn sam_hello(sock: &mut SamSocket) -> Result<()> {
    write_line(sock, &format!("HELLO VERSION MIN={SAM_MIN} MAX={SAM_MAX}")).await?;
    sam_check_ok(&read_line(sock).await?)?;
    Ok(())
}

async fn write_line(sock: &mut SamSocket, line: &str) -> Result<()> {
    sock.write_all(line.as_bytes()).await?;
    sock.write_all(b"\n").await?;
    sock.flush().await?;
    Ok(())
}

async fn read_line(sock: &mut SamSocket) -> Result<String> {
    let mut line = String::new();
    let n = sock.read_line(&mut line).await?;
    if n == 0 {
        bail!("SAM bridge closed the connection");
    }
    Ok(line)
}

/// Split a `dest[:port]` address into the I2P destination and an optional
/// stream port. The split only fires when the suffix after the last `:` is a
/// valid `u16` — base64 destinations never contain `:`, and a bare
/// `*.b32.i2p` / hostname has no port, so both pass through untouched.
fn split_dest_port(addr: &str) -> (&str, Option<u16>) {
    match addr.rsplit_once(':') {
        Some((dest, port)) => match port.parse::<u16>() {
            Ok(p) => (dest, Some(p)),
            Err(_) => (addr, None),
        },
        None => (addr, None),
    }
}

/// Split a SAM reply into tokens on whitespace, except inside double quotes (a
/// SAM `MESSAGE="no route to peer"` value may contain spaces). Quote characters
/// are dropped; the spaces they protect are kept.
fn sam_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in line.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// Tokenize a SAM reply line's `KEY=VALUE` pairs. Bare command words (e.g.
/// `HELLO REPLY`) have no `=` and are ignored. Values may have been quoted.
fn sam_kv(line: &str) -> HashMap<String, String> {
    sam_tokens(line)
        .into_iter()
        .filter_map(|tok| {
            tok.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect()
}

/// Parse a SAM reply and require `RESULT=OK`, returning its key/value pairs.
/// Surfaces the router's `RESULT` code (and `MESSAGE`, if any) on failure.
fn sam_check_ok(line: &str) -> Result<HashMap<String, String>> {
    let kv = sam_kv(line);
    match kv.get("RESULT").map(String::as_str) {
        Some("OK") => Ok(kv),
        Some(result) => match kv.get("MESSAGE") {
            Some(msg) => bail!("SAM error {result}: {msg}"),
            None => bail!("SAM error {result}"),
        },
        None => bail!("malformed SAM reply: {:?}", line.trim_end()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello_reply_ok() {
        let kv = sam_check_ok("HELLO REPLY RESULT=OK VERSION=3.3\n").unwrap();
        assert_eq!(kv.get("VERSION").unwrap(), "3.3");
    }

    #[test]
    fn extracts_naming_lookup_destination() {
        let kv = sam_check_ok("NAMING REPLY RESULT=OK NAME=ME VALUE=abcDEST123\n").unwrap();
        assert_eq!(kv.get("VALUE").unwrap(), "abcDEST123");
    }

    #[test]
    fn surfaces_result_code_and_message() {
        let err = sam_check_ok("STREAM STATUS RESULT=CANT_REACH_PEER MESSAGE=\"no route\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("CANT_REACH_PEER"), "got: {err}");
        assert!(err.contains("no route"), "got: {err}");
    }

    #[test]
    fn result_without_message_still_errors() {
        let err = sam_check_ok("SESSION STATUS RESULT=DUPLICATED_ID\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("DUPLICATED_ID"), "got: {err}");
    }

    #[test]
    fn missing_result_is_malformed() {
        assert!(sam_check_ok("HELLO REPLY VERSION=3.3\n").is_err());
    }

    #[test]
    fn splits_optional_stream_port() {
        assert_eq!(
            split_dest_port("peer.b32.i2p:7777"),
            ("peer.b32.i2p", Some(7777))
        );
        // a bare destination / hostname is untouched
        assert_eq!(split_dest_port("peer.b32.i2p"), ("peer.b32.i2p", None));
        // base64 dest with no port
        assert_eq!(split_dest_port("AAAABBBBcccc"), ("AAAABBBBcccc", None));
        // non-numeric suffix is not a port → leave the whole string as dest
        assert_eq!(split_dest_port("host:notaport"), ("host:notaport", None));
    }

    #[test]
    fn kv_ignores_bare_words_and_strips_quotes() {
        let kv = sam_kv("STREAM STATUS RESULT=OK MESSAGE=\"hi there\"");
        assert_eq!(kv.get("RESULT").unwrap(), "OK");
        assert_eq!(kv.get("MESSAGE").unwrap(), "hi there");
        assert!(!kv.contains_key("STREAM"));
    }
}
