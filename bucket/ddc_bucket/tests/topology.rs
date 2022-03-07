use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Topology {
    pub engine_name: String,
    pub partition_count: usize,
}

impl Topology {
    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn from_str(ser: &str) -> serde_json::Result<Topology> {
        serde_json::from_str(ser)
    }

    pub fn get_replica_indices(&self, routing_key: usize, replication: usize) -> Vec<usize> {
        (0..replication).map(|i|
            (routing_key + i) % self.partition_count
        ).collect()
    }
}
