use crate::ddc_bucket::*;

use super::node::TestNode;

pub struct TestStorage {
    pub vnode: TestNode,
}

impl TestStorage {
    pub fn new(provider_id: AccountId) -> Self {
        Self { vnode: TestNode::new(provider_id, "storage") }
    }

    pub fn handle_request(&self, _bucket_id: BucketId) {}
}
