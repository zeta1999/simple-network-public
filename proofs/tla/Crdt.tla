---- MODULE Crdt ----
EXTENDS Naturals, FiniteSets

CONSTANTS Elements, Tags

VARIABLES added, removed

TypeOK ==
    /\ added \in SUBSET (Elements \X Tags)
    /\ removed \in SUBSET Tags

Init == 
    /\ added = {}
    /\ removed = {}

Add(e, t) ==
    /\ added' = added \cup {<<e, t>>}
    /\ UNCHANGED removed

Remove(e) ==
    \* Finds all tags associated with e in added and adds them to removed
    /\ removed' = removed \cup {t \in Tags : <<e, t>> \in added}
    /\ UNCHANGED added

Merge(otherAdded, otherRemoved) ==
    /\ added' = added \cup otherAdded
    /\ removed' = removed \cup otherRemoved

Next == 
    \/ \E e \in Elements, t \in Tags : Add(e, t)
    \/ \E e \in Elements : Remove(e)

Spec == Init /\ [][Next]_<<added, removed>>
=====================
