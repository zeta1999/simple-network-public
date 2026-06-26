---- MODULE Dht ----
EXTENDS Naturals, Sequences

CONSTANTS K, MaxBuckets, Nodes

VARIABLES buckets

Init == buckets = [i \in 1..MaxBuckets |-> <<>>]

AddNode(n) ==
    \* Logical representation of XOR metric bucket assignment
    /\ UNCHANGED buckets

Next == \E n \in Nodes : AddNode(n)

Spec == Init /\ [][Next]_buckets
=====================
