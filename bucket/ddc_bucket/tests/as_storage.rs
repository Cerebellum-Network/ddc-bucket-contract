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

    pub fn handle_request(&mut self, _contract: &DdcBucket, request: &TestRequest) {
        assert_eq!(request.url, self.vnode.url, "wrong storage URL");

        // TODO: check bucket - cluster - node relationship.

        match &request.action {
            TestAction::Write(value) => {
                self.stored_data.insert(request.bucket_id, value.clone());
            }

            TestAction::Read(expected_value) => {
                let stored_value = self.stored_data
                    .get(&request.bucket_id)
                    .expect("No stored data for bucket");

                assert_eq!(stored_value, expected_value, "Incorrect stored data");
            }
        };
    }
}
