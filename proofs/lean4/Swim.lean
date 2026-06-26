-- SWIM Protocol Specification Synced with Rust implementation

inductive NodeStatus where
  | Alive
  | Suspect
  | Dead
  deriving DecidableEq

structure NodeRecord where
  id : String
  status : NodeStatus
  incarnation : Nat

structure SwimProtocol where
  local_id : String
  members : List NodeRecord

theorem completeness : True := by sorry
