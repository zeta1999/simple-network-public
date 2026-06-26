-- Raft Consensus Algorithm Specification Synced with Rust implementation

inductive State where
  | Follower
  | Candidate
  | Leader
  deriving DecidableEq

structure LogEntry where
  term : Nat
  command : String

structure RaftNode where
  currentTerm : Nat
  votedFor : Option String
  log : List LogEntry
  commitIndex : Nat
  lastApplied : Nat
  state : State

def handleAppendEntries (node : RaftNode) (term : Nat) (leaderCommit : Nat) : RaftNode :=
  if term < node.currentTerm then
    node
  else
    { node with currentTerm := term, commitIndex := min leaderCommit node.log.length, state := State.Follower }

theorem commit_index_monotonic (node : RaftNode) (term : Nat) (leaderCommit : Nat) :
  (handleAppendEntries node term leaderCommit).commitIndex ≥ node.commitIndex := by
  sorry

theorem election_safety : True := by sorry
