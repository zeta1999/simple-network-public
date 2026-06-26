---- MODULE Raft ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS Server, Value, Nil

VARIABLES currentTerm, state, votedFor, log, commitIndex, lastApplied

TypeOK == 
    /\ currentTerm \in [Server -> Nat]
    /\ state \in [Server -> {"Follower", "Candidate", "Leader"}]
    /\ votedFor \in [Server -> Server \cup {Nil}]
    /\ commitIndex \in [Server -> Nat]

Init == 
    /\ currentTerm = [i \in Server |-> 0]
    /\ state = [i \in Server |-> "Follower"]
    /\ votedFor = [i \in Server |-> Nil]
    /\ log = [i \in Server |-> <<>>]
    /\ commitIndex = [i \in Server |-> 0]
    /\ lastApplied = [i \in Server |-> 0]

RequestVote(i, j) ==
    \* Simple logical abstraction of request vote
    /\ currentTerm' = [currentTerm EXCEPT ![j] = IF currentTerm[i] > currentTerm[j] THEN currentTerm[i] ELSE currentTerm[j]]
    /\ UNCHANGED <<state, votedFor, log, commitIndex, lastApplied>>

AppendEntries(i, j) ==
    \* Simple logical abstraction of append entries
    /\ currentTerm' = [currentTerm EXCEPT ![j] = IF currentTerm[i] > currentTerm[j] THEN currentTerm[i] ELSE currentTerm[j]]
    /\ UNCHANGED <<state, votedFor, log, commitIndex, lastApplied>>

Next ==
    \/ \E i, j \in Server : RequestVote(i, j)
    \/ \E i, j \in Server : AppendEntries(i, j)

Spec == Init /\ [][Next]_<<currentTerm, state, votedFor, log, commitIndex, lastApplied>>

ElectionSafety == 
    \A i, j \in Server : (state[i] = "Leader" /\ state[j] = "Leader" /\ currentTerm[i] = currentTerm[j]) => (i = j)

=====================
