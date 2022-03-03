use crate::ddc_bucket::*;

use super::node::TestNode;
use crate::ddc_bucket::tests::node::TestRequest;


pub struct TestGateway {
    pub vnode: TestNode,
}

impl TestGateway {
    pub fn new(provider_id: AccountId, node_name: &str) -> Self {
        Self { vnode: TestNode::new(provider_id, "gateway", node_name) }
    }

    pub fn handle_request(&self, request: TestRequest) -> Result<()> {
        assert_eq!(request.url, self.vnode.url, "wrong URL");
        Ok(())
    }
}