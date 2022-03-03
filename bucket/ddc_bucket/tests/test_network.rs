use ink_lang as ink;

use crate::ddc_bucket::*;

use super::{as_gateway::*, as_storage::*, as_user::*, env_utils::*, node::*};

#[ink::test]
fn storage_network_works() {
    let accounts = get_accounts();

    let mut contract = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Create a storage Cluster and a gateway Cluster.
    push_caller(accounts.alice);
    let storage_cluster_id = contract.cluster_create(STORAGE_ENGINE.to_string())?;
    let gateway_cluster_id = contract.cluster_create(GATEWAY_ENGINE.to_string())?;
    pop_caller();

    // Provide one gateway VNode.
    let mut gateway_node = TestGateway::new(accounts.alice, "alice");
    gateway_node.vnode.join_cluster(&mut contract, gateway_cluster_id)?;

    // Provide two storage VNode’s.
    let mut storage_node0 = TestStorage::new(accounts.bob, "bob");
    storage_node0.vnode.join_cluster(&mut contract, storage_cluster_id)?;

    let mut storage_node1 = TestStorage::new(accounts.charlie, "charlie");
    storage_node1.vnode.join_cluster(&mut contract, storage_cluster_id)?;

    assert_ne!(storage_node0.vnode.url, storage_node1.vnode.url, "nodes must have different URLs");

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut contract, accounts.django)?;

    // Simulate a write request to the gateway.
    let data = "data";
    {
        let action = TestAction::Write(data.to_string());
        let request = user.make_request(&contract, action)?;
        let storage_requests = gateway_node.handle_request(&contract, request)?;

        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), 2);
        storage_node0.handle_request(&contract, &storage_requests[0])?;
        storage_node1.handle_request(&contract, &storage_requests[1])?;
    }

    // Simulate a read request to the gateway.
    {
        let action = TestAction::Read(data.to_string());
        let request = user.make_request(&contract, action)?;
        let storage_requests = gateway_node.handle_request(&contract, request)?;
        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), 2);
        storage_node0.handle_request(&contract, &storage_requests[0])?;
        storage_node1.handle_request(&contract, &storage_requests[1])?;
    }
}
