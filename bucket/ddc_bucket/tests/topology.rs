use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Topology {
    pub engine_name: String,
    pub vnode_count: usize,
    pub ring_tokens: Vec<u32>,
}

impl Topology {
    pub fn new(engine_name: &str, v_nodes: Vec<Vec<u64>>) -> Self {
        let ring_tokens = (1..1 + vnode_count)
            .map(|i| (u32::MAX / vnode_count) * i as u32)
            .collect();

        Self {
            engine_name: engine_name.to_string(),
            vnode_count: vnode_count as usize,
            ring_tokens,
        }
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn from_str(ser: &str) -> serde_json::Result<Self> {
        serde_json::from_str(ser)
    }

    pub fn get_replica_indices(&self, routing_key: u32, replication: u32) -> Vec<usize> {
        let first = self.get_segment_index(routing_key);

        (0..replication as usize)
            .map(|i| (first + i) % self.vnode_count)
            .collect()
    }

    pub fn get_segment_index(&self, routing_key: u32) -> usize {
        for (i, &token) in self.ring_tokens.iter().enumerate() {
            if routing_key < token {
                return i;
            }
        }
        return 0;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BucketParams {
    pub replication: u32,
}

impl BucketParams {
    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn from_str(ser: &str) -> serde_json::Result<Self> {
        serde_json::from_str(ser)
    }
}
