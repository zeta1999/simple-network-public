---- MODULE Gossip ----
EXTENDS Naturals, FiniteSets

CONSTANTS Nodes, Keys, Values

VARIABLES facts

Init == facts = [i \in Nodes |-> {}]

UpdateFact(n, k, v) ==
    /\ facts' = [facts EXCEPT ![n] = facts[n] \cup {<<k, v, 1>>}]

MergeFacts(n1, n2) ==
    /\ facts' = [facts EXCEPT ![n1] = facts[n1] \cup facts[n2]]

Next == 
    \/ \E n \in Nodes, k \in Keys, v \in Values : UpdateFact(n, k, v)
    \/ \E n1, n2 \in Nodes : MergeFacts(n1, n2)

Spec == Init /\ [][Next]_facts
=====================
