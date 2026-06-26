# PQC secure channel (`security::pqc`, feature `pqc`)

A mutually-authenticated, post-quantum confidential channel. Built on
`rust-secure-memory`; off by default (enable with `--features pqc`).

## What

- `Identity` — long-term **ML-DSA-65** signing identity; `verifying_key()` is
  what peers **pin** at pairing.
- `Initiator` / `Responder` — the two handshake roles. Each step is pure
  bytes-in/bytes-out, so it rides any `transport::Transport`.
- `SecureSession` — post-handshake `seal`/`open` of application records.

## Why

- **Post-quantum, today:** key agreement is hybrid **ML-KEM-768 + X25519**, so
  "harvest-now-decrypt-later" fails.
- **End-to-end:** confidentiality holds over an untrusted hop (e.g. a Tor
  relay) — TLS-to-the-relay alone would not.
- **Strong identity:** both sides ML-DSA-sign the handshake and verify against
  a pinned key → MITM- and quantum-resistant auth.

## How

```
client: Initiator::new(client_id, pinned_server_vk)
server: Responder::new(server_id, pinned_client_vk)

c→s   hello      = initiator.hello()            // ephemeral KEM pub, signed
s→c   (resp, S)  = responder.respond(hello)     // encapsulate, sign; server session
c     C          = initiator.finish(resp)       // verify+decapsulate; client session

S.seal(pt) / C.open(ct)   // XChaCha20-Poly1305 under a KDF'd KEM secret
```

Record key = `sequential_stretch(kem_secret, 1)` held in a `LockedBuffer`;
records use `crypto::{encrypt,decrypt}` (XChaCha20-Poly1305).

## When to use

Any link needing PQC confidentiality + pinned mutual auth without trusting the
transport — e.g. the `tutor-app-v0` client↔home-server channel over Tor (b1).
See `tutor-app-v0/docs/simple-network-requirements.md` §4.

## Tests

`cargo test -p simple_network --features pqc` covers: handshake → seal/open
round-trip (both directions), tampered ciphertext rejected, and pinned-identity
mismatch rejected on each side.

## Not yet (follow-ups)

- Bind the transcript into the KDF (hello+resp) for full channel binding.
- Forward secrecy across many records (rekey / ratchet).
- Wrap as a `SecurityPlugin` + integrate with `transport` (Tor) and pairing.
