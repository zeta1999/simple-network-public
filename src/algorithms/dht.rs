#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhtNode {
    pub node_id: u64,
    pub address: String,
}

pub struct Dht {
    pub local_id: u64,
    pub buckets: Vec<Vec<DhtNode>>,
    pub k: usize,
}

impl Dht {
    pub fn new(local_id: u64, k: usize) -> Self {
        let mut buckets = Vec::with_capacity(64);
        for _ in 0..64 {
            buckets.push(Vec::new());
        }
        Self {
            local_id,
            buckets,
            k,
        }
    }

    pub fn xor_distance(id1: u64, id2: u64) -> u64 {
        id1 ^ id2
    }

    pub fn bucket_index(local_id: u64, target_id: u64) -> usize {
        let distance = Self::xor_distance(local_id, target_id);
        if distance == 0 {
            return 0;
        }
        63 - distance.leading_zeros() as usize
    }

    pub fn add_node(&mut self, node: DhtNode) {
        if node.node_id == self.local_id {
            return;
        }
        let index = Self::bucket_index(self.local_id, node.node_id);
        let bucket = &mut self.buckets[index];

        bucket.retain(|n| n.node_id != node.node_id);
        bucket.push(node);

        if bucket.len() > self.k {
            bucket.remove(0);
        }
    }

    pub fn find_closest_nodes(&self, target_id: u64, count: usize) -> Vec<DhtNode> {
        let mut all_nodes = Vec::new();
        for bucket in &self.buckets {
            all_nodes.extend(bucket.iter().cloned());
        }

        all_nodes.sort_by_key(|n| Self::xor_distance(n.node_id, target_id));
        all_nodes.into_iter().take(count).collect()
    }
}
