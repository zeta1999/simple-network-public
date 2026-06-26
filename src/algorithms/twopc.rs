use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionState {
    Init,
    Prepared,
    Committed,
    Aborted,
}

pub struct TwoPhaseCommitCoordinator {
    pub tx_id: String,
    pub state: TransactionState,
    pub participants: Vec<String>,
    pub votes: HashMap<String, bool>,
}

impl TwoPhaseCommitCoordinator {
    pub fn new(tx_id: String, participants: Vec<String>) -> Self {
        Self {
            tx_id,
            state: TransactionState::Init,
            participants,
            votes: HashMap::new(),
        }
    }

    pub fn receive_vote(&mut self, participant: String, vote_commit: bool) -> Result<()> {
        if self.state != TransactionState::Init {
            return Err(anyhow::anyhow!("Transaction not in Init state"));
        }

        self.votes.insert(participant, vote_commit);

        if self.votes.values().any(|&v| !v) {
            self.state = TransactionState::Aborted;
            return Ok(());
        }

        if self.votes.len() == self.participants.len() && self.votes.values().all(|&v| v) {
            self.state = TransactionState::Prepared;
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        if self.state != TransactionState::Prepared {
            return Err(anyhow::anyhow!("Cannot commit, not prepared"));
        }
        self.state = TransactionState::Committed;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.state = TransactionState::Aborted;
    }
}

pub struct TwoPhaseCommitParticipant {
    pub tx_id: String,
    pub state: TransactionState,
}

impl TwoPhaseCommitParticipant {
    pub fn new(tx_id: String) -> Self {
        Self {
            tx_id,
            state: TransactionState::Init,
        }
    }

    pub fn prepare(&mut self, can_commit: bool) -> bool {
        if can_commit {
            self.state = TransactionState::Prepared;
            true
        } else {
            self.state = TransactionState::Aborted;
            false
        }
    }

    pub fn commit(&mut self) -> Result<()> {
        if self.state == TransactionState::Prepared {
            self.state = TransactionState::Committed;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot commit, not prepared"))
        }
    }

    pub fn abort(&mut self) {
        self.state = TransactionState::Aborted;
    }
}
