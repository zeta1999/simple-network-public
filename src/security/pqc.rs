//! Post-quantum secure channel (feature `pqc`).
//!
//! A mutually-authenticated, confidential byte channel built from
//! `rust-secure-memory`:
//!
//! - **Key agreement:** hybrid **ML-KEM-768 + X25519** (`HybridKemKeyPair` /
//!   `hybrid_kem::encapsulate`) — classical + post-quantum in one shot, so a
//!   harvest-now-decrypt-later attacker gains nothing.
//! - **Authentication:** **ML-DSA-65** signatures (`SigKeyPair`) over the
//!   handshake, each side verifying the other against a **pinned** verifying
//!   key (set at pairing) — MITM- and quantum-resistant identity.
//! - **Record protection:** **XChaCha20-Poly1305** (`crypto::{encrypt,decrypt}`)
//!   under a key derived from the KEM secret; key held in a `LockedBuffer`.
//!
//! The handshake is transport-agnostic: each step consumes/produces opaque
//! bytes the caller ships over any `Transport`. This is the b1 backbone for the
//! tutor client (see `tutor-app-v0/docs/simple-network-requirements.md`); it
//! also gives `tutor` true end-to-end security over an untrusted hop (Tor),
//! which TLS-to-the-relay alone could not.

use crate::transport::traits::Connection;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use secure_memory::hybrid_kem::{self, HybridKemKeyPair};
use secure_memory::sig::SigKeyPair;
use secure_memory::{crypto, sequential_stretch, LockedBuffer};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Long-term ML-DSA identity. The verifying key is what peers pin.
pub struct Identity {
    sig: SigKeyPair,
}

impl Identity {
    pub fn generate() -> Result<Identity> {
        Ok(Identity {
            sig: SigKeyPair::generate().map_err(to_anyhow)?,
        })
    }
    /// Public verifying key to share at pairing (peers pin this).
    pub fn verifying_key(&self) -> Vec<u8> {
        self.sig.verifying_key().to_vec()
    }
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        self.sig.sign(msg).map_err(to_anyhow)
    }

    /// Export `(signing_key, verifying_key)` raw bytes for persistence.
    /// The signing key is sensitive — protect it at rest.
    pub fn export(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let sk = self
            .sig
            .signing_key()
            .as_slice()
            .map_err(to_anyhow)?
            .to_vec();
        Ok((sk, self.sig.verifying_key().to_vec()))
    }

    /// Reload an identity from previously exported bytes.
    pub fn from_bytes(signing_key: &[u8], verifying_key: &[u8]) -> Result<Identity> {
        Ok(Identity {
            sig: SigKeyPair::from_bytes(signing_key, verifying_key).map_err(to_anyhow)?,
        })
    }
}

// --------------------------------------------------------------------------
// Pairing: out-of-band one-time secret authenticates a verifying-key exchange
// --------------------------------------------------------------------------

/// A fresh one-time pairing secret (hex). Show it as text / QR out of band; the
/// peer must present the same secret to pair. (QR payload = address + secret.)
pub fn random_secret() -> Result<String> {
    let buf = LockedBuffer::random(16).map_err(to_anyhow)?;
    Ok(to_hex(buf.as_slice().map_err(to_anyhow)?))
}

/// Iterations used to stretch the out-of-band pairing secret into the AEAD wrap
/// key. The pairing secret produced by [`random_secret`] is 128-bit random, so
/// it is not brute-forceable; the stretch additionally raises the cost of
/// guessing a *weak* caller-supplied secret. (For genuinely low-entropy secrets,
/// a memory-hard KDF would be stronger — see PLAN/DEPENDENCY_REVIEW.)
const PAIRING_STRETCH_ITERS: u64 = 200_000;

fn secret_key(secret: &str) -> [u8; 32] {
    sequential_stretch(secret.trim().as_bytes(), PAIRING_STRETCH_ITERS)
}

/// Exchange verifying keys over `conn`, authenticated by the shared one-time
/// `secret`: each side AEAD-wraps its vk under a key derived from the secret,
/// so a peer without the secret can't pair (decrypt fails). Returns the peer's
/// pinned verifying key. `initiator` controls send/recv order (one side true).
pub async fn pair_exchange(
    conn: &mut Box<dyn Connection>,
    id: &Identity,
    secret: &str,
    initiator: bool,
) -> Result<Vec<u8>> {
    let key = secret_key(secret);
    let sealed = crypto::encrypt(&key, &id.verifying_key()).map_err(to_anyhow)?;
    let peer = if initiator {
        conn.send(Bytes::from(sealed)).await?;
        let got = conn.recv().await?;
        crypto::decrypt(&key, &got)
    } else {
        let got = conn.recv().await?;
        let p = crypto::decrypt(&key, &got);
        conn.send(Bytes::from(sealed)).await?;
        p
    }
    .map_err(|_| anyhow!("pairing failed: wrong code or tampered exchange"))?;
    if peer.len() != 1952 {
        return Err(anyhow!("paired peer key has unexpected size"));
    }
    Ok(peer)
}

fn to_hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

/// Which end of the handshake a session belongs to. Determines which directional
/// key is used for sending vs receiving.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    Initiator,
    Responder,
}

/// Derives a directional 32-byte key from the KEM shared secret, domain-separated
/// by `label` so the client→server and server→client keys are independent.
///
/// The concatenated input and the returned key are wrapped in `Zeroizing` so the
/// secret material is actually wiped on drop (a plain `for`-loop store would be a
/// dead store that the optimizer may elide).
fn derive_dir_key(raw: &[u8], label: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut input = Zeroizing::new(Vec::with_capacity(raw.len() + label.len()));
    input.extend_from_slice(raw);
    input.extend_from_slice(label);
    Zeroizing::new(sequential_stretch(&input, 1))
}

const LABEL_C2S: &[u8] = b"simple-network/pqc/key/client-to-server/v1";
const LABEL_S2C: &[u8] = b"simple-network/pqc/key/server-to-client/v1";

/// An established channel: seal/open application records.
///
/// Each direction uses an independent key (so a record cannot be *reflected*
/// back to its sender and still decrypt) and a monotonic sequence number bound
/// into the AEAD as associated data (so a captured record cannot be *replayed*).
/// Records must therefore be processed in order — appropriate for the reliable,
/// in-order transports this is used over (TCP/TLS).
pub struct SecureSession {
    send_key: LockedBuffer,
    recv_key: LockedBuffer,
    send_seq: u64,
    recv_seq: u64,
}

impl SecureSession {
    fn from_secret(secret: &LockedBuffer, role: Role) -> Result<SecureSession> {
        let raw = secret.as_slice().map_err(to_anyhow)?;
        let c2s = derive_dir_key(raw, LABEL_C2S);
        let s2c = derive_dir_key(raw, LABEL_S2C);
        // `send`/`recv` are references into the Zeroizing keys above, which wipe
        // on drop; LockedBuffer copies into mlock'd, zeroized-on-drop storage.
        let (send, recv) = match role {
            Role::Initiator => (&c2s, &s2c),
            Role::Responder => (&s2c, &c2s),
        };
        Ok(SecureSession {
            send_key: LockedBuffer::from_bytes(&send[..]).map_err(to_anyhow)?,
            recv_key: LockedBuffer::from_bytes(&recv[..]).map_err(to_anyhow)?,
            send_seq: 0,
            recv_seq: 0,
        })
    }

    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let k = self.send_key.as_slice().map_err(to_anyhow)?;
        let aad = self.send_seq.to_le_bytes();
        let ct = crypto::encrypt_aad(k, plaintext, &aad).map_err(to_anyhow)?;
        self.send_seq = self
            .send_seq
            .checked_add(1)
            .ok_or_else(|| anyhow!("send sequence exhausted; rekey required"))?;
        Ok(ct)
    }

    pub fn open(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let k = self.recv_key.as_slice().map_err(to_anyhow)?;
        let aad = self.recv_seq.to_le_bytes();
        // A replayed or reordered record carries the wrong (already-consumed)
        // sequence number, so the AAD will not match and decryption fails.
        let pt = crypto::decrypt_aad(k, ciphertext, &aad).map_err(to_anyhow)?;
        self.recv_seq = self
            .recv_seq
            .checked_add(1)
            .ok_or_else(|| anyhow!("recv sequence exhausted; rekey required"))?;
        Ok(pt)
    }
}

// Wire messages (length-agnostic; JSON over the byte channel).
#[derive(Serialize, Deserialize)]
struct ClientHello {
    kem_pub: Vec<u8>,
    client_vk: Vec<u8>,
    sig: Vec<u8>, // ML-DSA over kem_pub
}

#[derive(Serialize, Deserialize)]
struct ServerResponse {
    ciphertext: Vec<u8>,
    server_vk: Vec<u8>,
    sig: Vec<u8>, // ML-DSA over ciphertext
}

/// Client side of the handshake. `peer_vk` is the server's pinned identity.
pub struct Initiator {
    kem: HybridKemKeyPair,
    id: Identity,
    peer_vk: Vec<u8>,
}

impl Initiator {
    pub fn new(id: Identity, peer_vk: Vec<u8>) -> Result<Initiator> {
        Ok(Initiator {
            kem: HybridKemKeyPair::generate().map_err(to_anyhow)?,
            id,
            peer_vk,
        })
    }

    /// First message: ephemeral KEM public key, signed by our identity.
    pub fn hello(&self) -> Result<Vec<u8>> {
        let kem_pub = self.kem.public_key();
        let sig = self.id.sign(&kem_pub)?;
        let hello = ClientHello {
            kem_pub,
            client_vk: self.id.verifying_key(),
            sig,
        };
        Ok(serde_json::to_vec(&hello)?)
    }

    /// Consume the server's response → an established session.
    pub fn finish(self, server_msg: &[u8]) -> Result<SecureSession> {
        let resp: ServerResponse = serde_json::from_slice(server_msg)?;
        check_pin(&resp.server_vk, &self.peer_vk)?;
        if !SigKeyPair::verify(&self.peer_vk, &resp.ciphertext, &resp.sig).map_err(to_anyhow)? {
            return Err(anyhow!("server signature invalid"));
        }
        let secret = self.kem.decapsulate(&resp.ciphertext).map_err(to_anyhow)?;
        SecureSession::from_secret(&secret, Role::Initiator)
    }
}

/// Server side. `peer_vk` is the client's pinned identity.
pub struct Responder {
    id: Identity,
    peer_vk: Vec<u8>,
}

impl Responder {
    pub fn new(id: Identity, peer_vk: Vec<u8>) -> Responder {
        Responder { id, peer_vk }
    }

    /// Consume the client hello → (response to send back, established session).
    pub fn respond(&self, client_hello: &[u8]) -> Result<(Vec<u8>, SecureSession)> {
        let hello: ClientHello = serde_json::from_slice(client_hello)?;
        check_pin(&hello.client_vk, &self.peer_vk)?;
        if !SigKeyPair::verify(&self.peer_vk, &hello.kem_pub, &hello.sig).map_err(to_anyhow)? {
            return Err(anyhow!("client signature invalid"));
        }
        let (ciphertext, secret) = hybrid_kem::encapsulate(&hello.kem_pub).map_err(to_anyhow)?;
        let sig = self.id.sign(&ciphertext)?;
        let resp = ServerResponse {
            ciphertext,
            server_vk: self.id.verifying_key(),
            sig,
        };
        let session = SecureSession::from_secret(&secret, Role::Responder)?;
        Ok((serde_json::to_vec(&resp)?, session))
    }
}

// --------------------------------------------------------------------------
// Secure channel over a transport `Connection`
// --------------------------------------------------------------------------

/// A `Connection` wrapped with an established PQC session: every `send` is
/// sealed and every `recv` is opened. (Inherent methods rather than the
/// `Connection` trait, since the AEAD key buffer is `Send` but not `Sync`.)
pub struct SecureConnection {
    inner: Box<dyn Connection>,
    session: SecureSession,
}

impl SecureConnection {
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        let ct = self.session.seal(data)?;
        self.inner.send(Bytes::from(ct)).await
    }
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        let ct = self.inner.recv().await?;
        self.session.open(&ct)
    }
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

/// Client: run the handshake over `conn`, pinning the server's identity.
pub async fn secure_client(
    mut conn: Box<dyn Connection>,
    id: Identity,
    server_vk: Vec<u8>,
) -> Result<SecureConnection> {
    let initiator = Initiator::new(id, server_vk)?;
    conn.send(Bytes::from(initiator.hello()?)).await?;
    let resp = conn.recv().await?;
    let session = initiator.finish(&resp)?;
    Ok(SecureConnection {
        inner: conn,
        session,
    })
}

/// Server: run the handshake over `conn`, pinning the client's identity.
pub async fn secure_server(
    mut conn: Box<dyn Connection>,
    id: Identity,
    client_vk: Vec<u8>,
) -> Result<SecureConnection> {
    let responder = Responder::new(id, client_vk);
    let hello = conn.recv().await?;
    let (resp, session) = responder.respond(&hello)?;
    conn.send(Bytes::from(resp)).await?;
    Ok(SecureConnection {
        inner: conn,
        session,
    })
}

fn check_pin(presented: &[u8], pinned: &[u8]) -> Result<()> {
    if presented != pinned {
        return Err(anyhow!("peer identity does not match pinned key"));
    }
    Ok(())
}

fn to_anyhow(e: secure_memory::Error) -> anyhow::Error {
    anyhow!("secure-memory: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full mutually-authenticated handshake, both sides pin the other.
    fn paired() -> Result<(SecureSession, SecureSession)> {
        let client_id = Identity::generate()?;
        let server_id = Identity::generate()?;
        let client_vk = client_id.verifying_key();
        let server_vk = server_id.verifying_key();

        let initiator = Initiator::new(client_id, server_vk)?;
        let responder = Responder::new(server_id, client_vk);

        let hello = initiator.hello()?;
        let (resp, server_sess) = responder.respond(&hello)?;
        let client_sess = initiator.finish(&resp)?;
        Ok((client_sess, server_sess))
    }

    #[test]
    fn handshake_then_seal_open_roundtrip() {
        let (mut client, mut server) = paired().unwrap();
        let ct = client.seal(b"hello over PQC").unwrap();
        assert_ne!(ct, b"hello over PQC");
        assert_eq!(server.open(&ct).unwrap(), b"hello over PQC");
        // and the other direction
        let ct2 = server.seal(b"reply").unwrap();
        assert_eq!(client.open(&ct2).unwrap(), b"reply");
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let (mut client, mut server) = paired().unwrap();
        let mut ct = client.seal(b"secret").unwrap();
        let last = ct.len() - 1;
        ct[last] ^= 0xff;
        assert!(server.open(&ct).is_err());
    }

    #[test]
    fn replayed_record_is_rejected() {
        // A captured ciphertext must not decrypt a second time: its sequence
        // number is bound as AAD and the receiver has already advanced past it.
        let (mut client, mut server) = paired().unwrap();
        let ct = client.seal(b"transfer $100").unwrap();
        assert_eq!(server.open(&ct).unwrap(), b"transfer $100");
        assert!(
            server.open(&ct).is_err(),
            "replay of the same record must be rejected"
        );
    }

    #[test]
    fn reflected_record_is_rejected() {
        // A record the client sent (client→server key) must not open on the
        // client's own receive side (server→client key) — defeats reflection.
        let (mut client, mut server) = paired().unwrap();
        let ct = client.seal(b"to server only").unwrap();
        assert!(
            client.open(&ct).is_err(),
            "a record must not be openable by its own sender"
        );
        // sanity: it still opens correctly on the real peer
        assert_eq!(server.open(&ct).unwrap(), b"to server only");
    }

    #[test]
    fn out_of_order_records_are_rejected() {
        let (mut client, mut server) = paired().unwrap();
        let _r0 = client.seal(b"first").unwrap();
        let r1 = client.seal(b"second").unwrap();
        // Delivering record #1 before record #0 desyncs the sequence → reject.
        assert!(server.open(&r1).is_err());
    }

    #[test]
    fn wrong_pinned_identity_rejected() {
        // Client pins the WRONG server key → finish must fail.
        let client_id = Identity::generate().unwrap();
        let server_id = Identity::generate().unwrap();
        let impostor = Identity::generate().unwrap();
        let client_vk = client_id.verifying_key();

        let initiator = Initiator::new(client_id, impostor.verifying_key()).unwrap();
        let responder = Responder::new(server_id, client_vk);

        let hello = initiator.hello().unwrap();
        let (resp, _s) = responder.respond(&hello).unwrap();
        assert!(initiator.finish(&resp).is_err());
    }

    #[tokio::test]
    async fn pqc_over_tcp_roundtrip() {
        use crate::transport::tcp::TcpTransport;
        use crate::transport::traits::Transport;

        let server_id = Identity::generate().unwrap();
        let client_id = Identity::generate().unwrap();
        let server_vk = server_id.verifying_key();
        let client_vk = client_id.verifying_key();

        let t = TcpTransport;
        let mut listener = t.bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let conn = listener.accept().await.unwrap();
            let mut sc = secure_server(conn, server_id, client_vk).await.unwrap();
            assert_eq!(sc.recv().await.unwrap(), b"ping");
            sc.send(b"pong").await.unwrap();
        });

        let conn = t.connect(&addr.to_string()).await.unwrap();
        let mut cc = secure_client(conn, client_id, server_vk).await.unwrap();
        cc.send(b"ping").await.unwrap();
        assert_eq!(cc.recv().await.unwrap(), b"pong");
        server.await.unwrap();
    }

    #[test]
    fn identity_export_import_roundtrip() {
        let id = Identity::generate().unwrap();
        let vk = id.verifying_key();
        let (sk, vk2) = id.export().unwrap();
        assert_eq!(vk, vk2);
        let id2 = Identity::from_bytes(&sk, &vk2).unwrap();
        assert_eq!(id2.verifying_key(), vk);
    }

    #[tokio::test]
    async fn pairing_over_tcp_exchanges_pinned_keys() {
        use crate::transport::tcp::TcpTransport;
        use crate::transport::traits::Transport;

        let server_id = Identity::generate().unwrap();
        let client_id = Identity::generate().unwrap();
        let server_vk = server_id.verifying_key();
        let client_vk = client_id.verifying_key();
        let secret = random_secret().unwrap();
        let bad = random_secret().unwrap();

        let t = TcpTransport;
        let mut listener = t.bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let s = secret.clone();
        let server = tokio::spawn(async move {
            let mut conn = listener.accept().await.unwrap();
            pair_exchange(&mut conn, &server_id, &s, false).await
        });

        let mut conn = t.connect(&addr.to_string()).await.unwrap();
        let got_server_vk = pair_exchange(&mut conn, &client_id, &secret, true)
            .await
            .unwrap();
        let got_client_vk = server.await.unwrap().unwrap();

        assert_eq!(got_server_vk, server_vk);
        assert_eq!(got_client_vk, client_vk);
        assert_ne!(secret, bad);
    }

    #[test]
    fn unpinned_client_rejected_by_server() {
        let client_id = Identity::generate().unwrap();
        let server_id = Identity::generate().unwrap();
        let someone_else = Identity::generate().unwrap();

        // server pins someone else, not this client
        let initiator = Initiator::new(client_id, server_id.verifying_key()).unwrap();
        let responder = Responder::new(server_id, someone_else.verifying_key());

        let hello = initiator.hello().unwrap();
        assert!(responder.respond(&hello).is_err());
    }
}
