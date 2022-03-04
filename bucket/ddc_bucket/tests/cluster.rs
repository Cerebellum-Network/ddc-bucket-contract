use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Topology {
    pub engine_name: String,
    pub shard_count: usize,
    pub replica_count: usize,
}

impl Topology {
    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn from_str(ser: &str) -> serde_json::Result<Topology> {
        serde_json::from_str(ser)
    }

    pub fn get_replica_indices(&self, routing_key: usize) -> Vec<usize> {
        // Simulate routing.
        let shard_index = routing_key % self.shard_count;
        let first = shard_index * self.replica_count;
        // Find all the replicas for this shard.
        let indices = first..first + self.replica_count;
        indices.collect()
    }
}
