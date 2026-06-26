---- MODULE VectorClock ----
EXTENDS Naturals, FiniteSets, Functions

CONSTANTS Nodes

VARIABLES clocks

TypeOK == 
    /\ clocks \in [Nodes -> [Nodes -> Nat]]

Init == 
    /\ clocks = [i \in Nodes |-> [j \in Nodes |-> 0]]

Increment(n) ==
    /\ clocks' = [clocks EXCEPT ![n][n] = clocks[n][n] + 1]

Merge(n1, n2) ==
    /\ clocks' = [clocks EXCEPT ![n1] = [j \in Nodes |-> IF clocks[n1][j] > clocks[n2][j] THEN clocks[n1][j] ELSE clocks[n2][j]]]

Next == 
    \/ \E n \in Nodes : Increment(n)
    \/ \E n1, n2 \in Nodes : Merge(n1, n2)

Spec == Init /\ [][Next]_clocks

=====================
