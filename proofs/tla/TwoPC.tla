---- MODULE TwoPC ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS Participants

VARIABLES rmState, tmState, votes

Init == 
    /\ rmState = [p \in Participants |-> "Init"]
    /\ tmState = "Init"
    /\ votes = [p \in Participants |-> "None"]

RmPrepare(p, vote) ==
    /\ rmState[p] = "Init"
    /\ rmState' = [rmState EXCEPT ![p] = IF vote THEN "Prepared" ELSE "Aborted"]
    /\ votes' = [votes EXCEPT ![p] = IF vote THEN "Yes" ELSE "No"]
    /\ UNCHANGED tmState

TmCommit ==
    /\ tmState = "Init"
    /\ \A p \in Participants : votes[p] = "Yes"
    /\ tmState' = "Committed"
    /\ UNCHANGED <<rmState, votes>>

TmAbort ==
    /\ tmState = "Init"
    /\ \E p \in Participants : votes[p] = "No"
    /\ tmState' = "Aborted"
    /\ UNCHANGED <<rmState, votes>>

Next == 
    \/ \E p \in Participants, v \in BOOLEAN : RmPrepare(p, v)
    \/ TmCommit
    \/ TmAbort

Spec == Init /\ [][Next]_<<rmState, tmState, votes>>

=====================
