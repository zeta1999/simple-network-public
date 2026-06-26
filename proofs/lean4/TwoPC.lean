-- Two Phase Commit Specification Synced with Rust implementation

inductive TransactionState where
  | Init
  | Prepared
  | Committed
  | Aborted

structure Coordinator where
  state : TransactionState
  votes_received : Nat
  total_participants : Nat

structure Participant where
  state : TransactionState

theorem safety_property : True := by sorry
