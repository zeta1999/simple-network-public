-- Gossip Protocol Specification Synced with Rust implementation

structure GossipFact where
  key : String
  value : String
  version : Nat

structure GossipProtocol where
  node_id : String
  facts : List (String × GossipFact)
  peers : List String

theorem eventual_consistency : True := by sorry
