# Integration request — quantum multi-repo extension

> Deferred: only starts once the `quantum` repo is demo-ready. Master plan:
> `../../quantum/future/PLAN-AHEAD.md`; full asks: `../../quantum/future/requests/REQUESTS.md`.

The `llm-inference-tune` harness wants from `simple-network`:
- A reusable **AF_UNIX + websocket/QUIC server** with pluggable **auth-token middleware** and a
  **PQC-pairing hook** (`src/security/pairing.rs`).
- A stable **C/Go FFI ABI** (`src/ffi`) to bind the harness; mark the exact `Transport`/`Listener`
  traits `pub`.

No work required now — this records the intended integration surface.
