use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone)]
pub struct RaftLogEntry {
    pub term: u64,
    pub command: Vec<u8>,
}

pub struct RaftNode {
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub log: Vec<RaftLogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state: RaftState,
}

impl Default for RaftNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RaftNode {
    pub fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: vec![],
            commit_index: 0,
            last_applied: 0,
            state: RaftState::Follower,
        }
    }

    pub fn handle_append_entries(
        &mut self,
        term: u64,
        _prev_log_index: u64,
        _prev_log_term: u64,
        entries: Vec<RaftLogEntry>,
        leader_commit: u64,
    ) -> Result<bool> {
        if term < self.current_term {
            return Ok(false);
        }

        self.current_term = term;
        self.state = RaftState::Follower;

        self.log.extend(entries);

        if leader_commit > self.commit_index {
            self.commit_index = std::cmp::min(leader_commit, self.log.len() as u64);
        }

        Ok(true)
    }

    pub fn handle_request_vote(
        &mut self,
        term: u64,
        candidate_id: String,
        _last_log_index: u64,
        _last_log_term: u64,
    ) -> Result<bool> {
        if term < self.current_term {
            return Ok(false);
        }

        if term > self.current_term {
            self.current_term = term;
            self.state = RaftState::Follower;
            self.voted_for = None;
        }

        if self.voted_for.is_none() || self.voted_for.as_ref() == Some(&candidate_id) {
            self.voted_for = Some(candidate_id);
            return Ok(true);
        }

        Ok(false)
    }
}
