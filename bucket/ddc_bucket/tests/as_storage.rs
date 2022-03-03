use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::node::TestRequest;

use super::node::TestNode;

pub const STORAGE_ENGINE: &str = "storage";

pub struct TestStorage {
    pub vnode: TestNode,
}

impl TestStorage {
    pub fn new(provider_id: AccountId, node_name: &str) -> Self {
        Self { vnode: TestNode::new(provider_id, STORAGE_ENGINE, node_name) }
    }

    pub fn handle_request(&self, _contract: &DdcBucket, request: &TestRequest) {
        assert_eq!(request.url, self.vnode.url, "wrong storage URL");


    }
}
