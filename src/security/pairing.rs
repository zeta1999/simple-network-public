//! Legacy mTLS pairing skeleton.
//!
//! ⚠️ This module is **not implemented**. It previously returned hard-coded
//! `MOCKED_KEY_*` / `MOCKED_CERT` strings and printed them to stdout, which made
//! callers believe they had paired and obtained real key material when they had
//! not. It now fails loudly instead of returning plausible garbage.
//!
//! For real, post-quantum mutual authentication use [`crate::security::pqc`]
//! (`Identity` + `pair_exchange` + the pinned `Initiator`/`Responder`
//! handshake), which is implemented and tested.

use anyhow::{bail, Result};

pub struct PairingManager;

impl PairingManager {
    pub async fn mtls_pairing_procedure(_node_name: &str) -> Result<(String, String)> {
        bail!("mTLS pairing is not implemented; use security::pqc for authenticated pairing")
    }

    pub async fn verify_and_sign_csr(_csr: &str) -> Result<String> {
        bail!("CSR signing is not implemented; no CA is wired up")
    }

    pub async fn cluster_pairing_procedure(_node_name: &str) -> Result<()> {
        bail!("cluster pairing is not implemented; use security::pqc for authenticated pairing")
    }
}
