use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipFact {
    pub key: String,
    pub value: String,
    pub version: u64,
}

pub struct GossipProtocol {
    pub node_id: String,
    pub facts: HashMap<String, GossipFact>,
    pub peers: HashSet<String>,
}

impl GossipProtocol {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            facts: HashMap::new(),
            peers: HashSet::new(),
        }
    }

    pub fn add_peer(&mut self, peer_id: String) {
        self.peers.insert(peer_id);
    }

    pub fn update_fact(&mut self, key: String, value: String) {
        let version = self.facts.get(&key).map(|f| f.version).unwrap_or(0) + 1;
        self.facts.insert(
            key.clone(),
            GossipFact {
                key,
                value,
                version,
            },
        );
    }

    pub fn get_facts_for_sync(&self) -> Vec<GossipFact> {
        self.facts.values().cloned().collect()
    }

    pub fn merge_facts(&mut self, incoming_facts: Vec<GossipFact>) {
        for incoming in incoming_facts {
            let current = self.facts.get(&incoming.key);
            if current.is_none() || incoming.version > current.unwrap().version {
                self.facts.insert(incoming.key.clone(), incoming);
            }
        }
    }
}
