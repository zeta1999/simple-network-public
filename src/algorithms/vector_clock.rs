use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    pub node_id: String,
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new(node_id: String) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(node_id.clone(), 0);
        Self { node_id, clocks }
    }

    pub fn increment(&mut self) {
        let count = self.clocks.entry(self.node_id.clone()).or_insert(0);
        *count += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &clock) in &other.clocks {
            let my_clock = self.clocks.entry(node.clone()).or_insert(0);
            *my_clock = std::cmp::max(*my_clock, clock);
        }
    }

    pub fn precedes(&self, other: &VectorClock) -> bool {
        let mut strictly_smaller = false;

        for (node, &my_clock) in &self.clocks {
            let other_clock = other.clocks.get(node).copied().unwrap_or(0);
            if my_clock > other_clock {
                return false;
            }
            if my_clock < other_clock {
                strictly_smaller = true;
            }
        }

        for node in other.clocks.keys() {
            if !self.clocks.contains_key(node) {
                strictly_smaller = true;
            }
        }

        strictly_smaller
    }
}
