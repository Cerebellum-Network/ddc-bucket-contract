use crate::ddc_bucket::*;

use super::node::TestNode;


pub struct TestGateway {
    pub vnode: TestNode,
}

impl TestGateway {
    pub fn new(provider_id: AccountId) -> Self {
        Self { vnode: TestNode::new(provider_id, "gateway") }
    }

    pub fn handle_request(&self, _bucket_id: BucketId) {}
}