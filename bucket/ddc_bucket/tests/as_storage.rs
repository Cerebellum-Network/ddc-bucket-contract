use std::collections::HashMap;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::node::{TestAction, TestRequest};

use super::node::TestNode;

pub const STORAGE_ENGINE: &str = "storage";

pub struct TestStorage {
    pub vnode: TestNode,
    pub stored_data: HashMap<BucketId, String>,
}

impl TestStorage {
    pub fn new(provider_id: AccountId, node_name: &str) -> Self {
        Self {
            vnode: TestNode::new(provider_id, STORAGE_ENGINE, node_name),
            stored_data: Default::default(),
        }
    }

    pub fn handle_request(&mut self, contract: &DdcBucket, request: &TestRequest) -> Result<()> {
        assert_eq!(request.url, self.vnode.url, "wrong storage URL");

        // Fetch the status of this bucket.
        let status = contract.bucket_get_status(request.bucket_id)?;
        let cluster_id = status.bucket.cluster_ids.first().expect("bucket has no clusters");

        // Check that this bucket is allocated in the storage cluster of this node.
        let allocated = self.vnode.cluster_ids.contains(cluster_id);
        assert!(allocated, "bucket is not allocated on this node");

        match &request.action {
            TestAction::Write(value) => {
                // Check the writer permission.
                let authorized = status.writer_ids.contains(&request.sender);
                assert!(authorized, "sender is not authorized to write to this bucket");

                self.stored_data.insert(request.bucket_id, value.clone());
            }

            TestAction::Read(expected_value) => {
                let stored_value = self.stored_data
                    .get(&request.bucket_id)
                    .expect("No stored data for bucket");

                assert_eq!(stored_value, expected_value, "Incorrect stored data");
            }
        };
        Ok(())
    }
}
