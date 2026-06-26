//! Cluster mTLS transport builder.
//!
//! ⚠️ **Not implemented.** The previous version of `build` ignored its
//! certificate/key/CA arguments, left the trust root store empty, called
//! `with_no_client_auth()` on both ends, and passed an empty cert chain + empty
//! private key — i.e. it constructed a transport that performs *no* client
//! authentication (despite the "mTLS cluster" name) and cannot complete a
//! handshake. Returning such a config as if it were valid is worse than failing,
//! so `build` now returns an explicit error until real mTLS is wired up.

use super::tls::TlsTransport;
use anyhow::{bail, Result};

pub struct ClusterTransportBuilder;

impl ClusterTransportBuilder {
    /// Intended to build a mutually-authenticated (mTLS) cluster transport from
    /// DER-encoded node cert/key and the cluster CA. **Not yet implemented** —
    /// see the module docs. A real implementation must parse the DER inputs,
    /// populate the root store from `cluster_ca_der`, require client certs
    /// server-side (e.g. `WebPkiClientVerifier`), and present the node cert+key.
    pub fn build(
        _node_cert_der: Vec<u8>,
        _node_key_der: Vec<u8>,
        _cluster_ca_der: Vec<u8>,
        _domain: String,
    ) -> Result<TlsTransport> {
        bail!(
            "ClusterTransportBuilder::build is not implemented: mTLS cluster transport \
             (cert parsing, CA trust root, client-auth verifier) is not yet wired up"
        )
    }
}
