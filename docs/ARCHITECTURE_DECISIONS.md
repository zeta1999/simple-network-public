# Architecture Decisions

This document records the major architectural choices and user preferences for the `simple-network` project.

## 1. Distributed Algorithms Implementation
- **Decision:** All distributed algorithms (Raft, 2PC, Gossip, SWIM, Vector Clocks, CRDTs, DHT/Kademlia) must be implemented **from scratch**.
- **Rationale:** Ensures complete control over the networking semantics, tight integration with our custom `Transport` traits, and exact alignment with our verification goals.
- **Verification:** Each algorithm must be accompanied by skeleton proofs in **Lean4** and **TLA+**. The description and logic of the algorithms in these proofs must match the Rust implementations identically.

## 2. Erlang-Style Network Patterns
- **Decision:** The network primitives will closely mirror Erlang's robust OTP paradigms.
- **Implemented Patterns:** `gen_server` (RPC), `gen_statem` (State Machine), `pubsub`, `pipeline` (Push/Pull), `router_dealer`, and `net_kernel` (Clustering).

## 3. Workflow & CI
- **Decision:** We commit locally often, establishing a clear history for each phase, but we **do not push** to a remote automatically.
- **Tooling:** A top-level `./scripts/build-and-test-all.sh` orchestrates the local CI.

## 4. Security & Connections
- **Decision:** The encryption architecture is pluggable (`SecurityPlugin`), with default emphasis on mTLS for external connections and robust certificate-based pairing for internal cluster connections (`net_kernel`).
