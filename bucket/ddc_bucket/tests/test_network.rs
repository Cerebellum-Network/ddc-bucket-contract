use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::cluster::Topology;

use super::{as_gateway::*, as_storage::*, as_user::*, env_utils::*, node::*};

#[ink::test]
fn storage_network_works() {
    let accounts = get_accounts();

    let mut contract = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Create a storage Cluster and a gateway Cluster.
    push_caller(accounts.alice);

    let storage_cluster_id = {
        let topology = Topology {
            engine_name: STORAGE_ENGINE.to_string(),
            shard_count: 2,
            replica_count: 2,
        };
        contract.cluster_create(topology.to_string().unwrap())?
    };
    let gateway_cluster_id = {
        let topology = Topology {
            engine_name: GATEWAY_ENGINE.to_string(),
            shard_count: 1,
            replica_count: 1,
        };
        contract.cluster_create(topology.to_string().unwrap())?
    };

    pop_caller();

    // Provide one gateway VNode.
    let mut gateway_node = TestGateway::new(accounts.alice, "alice");
    gateway_node.vnode.join_cluster(&mut contract, gateway_cluster_id)?;

    // Provide storage VNodes.
    let node_specs = vec![
        (accounts.charlie, "charlie"),
        (accounts.django, "django"),
        (accounts.eve, "eve"),
        (accounts.frank, "frank"),
    ];
    let mut storage_nodes: Vec<TestStorage> =
        node_specs.iter().map(|spec| {
            let mut node = TestStorage::new(spec.0, spec.1);
            node.vnode.join_cluster(&mut contract, storage_cluster_id).unwrap();
            node
        }).collect();

    assert_ne!(storage_nodes[0].vnode.url, storage_nodes[1].vnode.url, "nodes must have different URLs");

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut contract, accounts.bob)?;

    // Simulate a write request to the gateway.
    let data = "data";
    {
        let action = TestAction::Write(data.to_string());
        let request = user.make_request(&contract, action)?;
        let storage_requests = gateway_node.handle_request(&contract, request)?;

        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), 2);
        storage_nodes[0].handle_request(&contract, &storage_requests[0])?;
        storage_nodes[1].handle_request(&contract, &storage_requests[1])?;
    }

    // Simulate a read request to the gateway.
    {
        let action = TestAction::Read(data.to_string());
        let request = user.make_request(&contract, action)?;
        let storage_requests = gateway_node.handle_request(&contract, request)?;
        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), 2);
        storage_nodes[0].handle_request(&contract, &storage_requests[0])?;
        storage_nodes[1].handle_request(&contract, &storage_requests[1])?;
    }
}
