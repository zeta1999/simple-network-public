---- MODULE Swim ----
EXTENDS Naturals, Sequences

CONSTANTS Nodes

VARIABLES members

Init == members = [i \in Nodes |-> [status |-> "Alive", incarnation |-> 0]]

UpdateMember(n, s, inc) ==
    /\ members' = [members EXCEPT ![n] = [status |-> s, incarnation |-> inc]]

Next == \E n \in Nodes, s \in {"Alive", "Suspect", "Dead"}, inc \in Naturals : UpdateMember(n, s, inc)

Spec == Init /\ [][Next]_members
=====================
