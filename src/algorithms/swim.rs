use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    Alive,
    Suspect,
    Dead,
}

#[derive(Debug, Clone)]
pub struct NodeRecord {
    pub id: String,
    pub status: NodeStatus,
    pub incarnation: u64,
    pub last_update: Instant,
}

pub struct SwimProtocol {
    pub local_id: String,
    pub members: HashMap<String, NodeRecord>,
    pub suspect_timeout: Duration,
}

impl SwimProtocol {
    pub fn new(local_id: String) -> Self {
        Self {
            local_id,
            members: HashMap::new(),
            suspect_timeout: Duration::from_secs(5),
        }
    }

    pub fn update_member(&mut self, id: String, status: NodeStatus, incarnation: u64) {
        if id == self.local_id {
            return;
        }

        let current = self.members.get(&id);
        if let Some(record) = current {
            if incarnation > record.incarnation
                || (incarnation == record.incarnation
                    && status == NodeStatus::Dead
                    && record.status != NodeStatus::Dead)
            {
                self.members.insert(
                    id.clone(),
                    NodeRecord {
                        id,
                        status,
                        incarnation,
                        last_update: Instant::now(),
                    },
                );
            }
        } else {
            self.members.insert(
                id.clone(),
                NodeRecord {
                    id,
                    status,
                    incarnation,
                    last_update: Instant::now(),
                },
            );
        }
    }

    pub fn check_timeouts(&mut self) {
        let now = Instant::now();
        for record in self.members.values_mut() {
            if record.status == NodeStatus::Suspect
                && now.duration_since(record.last_update) > self.suspect_timeout
            {
                record.status = NodeStatus::Dead;
                record.last_update = now;
            }
        }
    }

    pub fn get_alive_members(&self) -> Vec<String> {
        self.members
            .values()
            .filter(|r| r.status == NodeStatus::Alive)
            .map(|r| r.id.clone())
            .collect()
    }
}
