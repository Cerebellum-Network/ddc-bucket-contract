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

    let vnode_specs = vec![
        (accounts.charlie, "charlie-0"),
        (accounts.django, "django-0"),
        (accounts.eve, "eve-0"),
        (accounts.charlie, "charlie-1"),
        (accounts.django, "django-1"),
        (accounts.eve, "eve-1"),
    ];

    let storage_cluster_id = {
        let topology = Topology {
            engine_name: STORAGE_ENGINE.to_string(),
            partition_count: vnode_specs.len(),
        };
        contract.cluster_create(topology.to_string().unwrap())?
    };
    let gateway_cluster_id = {
        let topology = Topology {
            engine_name: GATEWAY_ENGINE.to_string(),
            partition_count: 1,
        };
        contract.cluster_create(topology.to_string().unwrap())?
    };

    pop_caller();

    // Provide one gateway VNode.
    let mut gateway_node = TestGateway::new(accounts.alice, "alice");
    gateway_node.vnode.join_cluster(&mut contract, gateway_cluster_id)?;

    // Provide storage VNodes.

    let mut storage_nodes: Vec<TestStorage> =
        vnode_specs.iter().map(|spec| {
            let mut node = TestStorage::new(spec.0, spec.1);
            node.vnode.join_cluster(&mut contract, storage_cluster_id).unwrap();
            node
        }).collect();

    assert_ne!(storage_nodes[0].vnode.url, storage_nodes[1].vnode.url, "nodes must have different URLs");

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut contract, accounts.bob)?;

    let mut execute_action = |action: Action, expect_nodes: &[usize]| {
        let request = user.make_request(&contract, action).unwrap();
        let storage_requests = gateway_node.handle_request(&contract, request).unwrap();

        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), expect_nodes.len());
        for (request_i, &node_i) in expect_nodes.iter().enumerate() {
            storage_nodes[node_i].handle_request(&contract, &storage_requests[request_i]).unwrap();
        }
    };

    // Simulate write requests to the gateway into different shards.
    execute_action(
        Action { routing_key: 0, data: "data in shard 0".to_string(), op: Op::Write },
        &[0, 1]);
    execute_action(
        Action { routing_key: 1, data: "data in shard 1".to_string(), op: Op::Write },
        &[1, 2]);
    execute_action(
        Action { routing_key: 4, data: "data in shard 4".to_string(), op: Op::Write },
        &[4, 5]);

    // Simulate read requests to the gateway.
    execute_action(
        Action { routing_key: 0, data: "data in shard 0".to_string(), op: Op::Read },
        &[0, 1]);
    execute_action(
        Action { routing_key: 1, data: "data in shard 1".to_string(), op: Op::Read },
        &[1, 2]);
    execute_action(
        Action { routing_key: 4, data: "data in shard 4".to_string(), op: Op::Read },
        &[4, 5]);
}
