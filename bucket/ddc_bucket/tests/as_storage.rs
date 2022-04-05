use crate::ddc_bucket::*;

use super::node::{Op, TestNode, TestRequest};

pub const STORAGE_ENGINE: &str = "storage";

pub struct TestStorage {
    pub node: TestNode,
    stored_data: Vec<Row>,
}

struct Row {
    bucket_id: BucketId,
    routing_key: u32,
    data: String,
}

impl TestStorage {
    pub fn new(contract: &mut DdcBucket, provider_id: AccountId, manager_id: AccountId, node_name: &str) -> Self {
        Self {
            node: TestNode::new(contract, provider_id, manager_id, STORAGE_ENGINE, node_name),
            stored_data: Default::default(),
        }
    }

    pub fn handle_request(&mut self, contract: &DdcBucket, request: &TestRequest) -> Result<()> {
        assert_eq!(request.url, self.node.url, "wrong storage URL");

        // Fetch the status of this bucket.
        let status = contract.bucket_get(request.bucket_id)?;
        let cluster = contract.cluster_get(status.bucket.cluster_id).unwrap().cluster;

        // Check that this bucket is allocated in the storage cluster of this node.
        let allocated = cluster.vnodes.contains(&self.node.node_id);
        assert!(allocated, "bucket is not allocated on this node");

        let bucket_id = request.bucket_id;
        let routing_key = request.action.routing_key;
        let data = request.action.data.clone();

        match &request.action.op {
            Op::Write => {
                // Check the writer permission.
                let authorized = status.writer_ids.contains(&request.sender);
                assert!(authorized, "sender is not authorized to write to this bucket");

                self.stored_data.push(
                    Row { bucket_id, routing_key, data });
            }

            Op::Read => {
                let stored_value = self.stored_data.iter()
                    .rfind(|row| {
                        row.bucket_id == bucket_id && row.routing_key == routing_key
                    })
                    .expect("No stored data for bucket");

                assert_eq!(stored_value.data, data, "Incorrect stored data");
            }
        };
        Ok(())
    }
}
