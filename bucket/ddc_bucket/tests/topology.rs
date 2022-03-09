use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Topology {
    pub engine_name: String,
    pub partition_count: u32,
    pub ring_tokens: Vec<u32>,
}

impl Topology {
    pub fn new(engine_name: &str, partition_count: u32) -> Self {
        let ring_tokens = (0..partition_count).map(|i| {
            u32::MAX / partition_count * i
        }).collect();

        Self {
            engine_name: engine_name.to_string(),
            partition_count,
            ring_tokens,
        }
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn from_str(ser: &str) -> serde_json::Result<Self> {
        serde_json::from_str(ser)
    }

    pub fn get_replica_indices(&self, routing_key: u32, replication: u32) -> Vec<u32> {
        (0..replication).map(|i|
            (routing_key + i) % self.partition_count
        ).collect()
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