use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::as_cluster_manager::ClusterManager;

use super::{as_gateway::*, as_storage::*, as_user::*, env_utils::*, node::*};

#[ink::test]
fn storage_network_works() {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    set_balance(accounts.django, 1000 * TOKEN);
    set_balance(accounts.eve, 1000 * TOKEN);
    let manager_id = accounts.alice;

    let mut contract = DdcBucket::new();

    let node_specs = vec![
        (accounts.charlie, "charlie-0"),
        (accounts.django, "django-0"),
        (accounts.eve, "eve-0"),
        (accounts.charlie, "charlie-1"),
        (accounts.django, "django-1"),
        (accounts.eve, "eve-1"),
    ];
    let vnode_count = node_specs.len() as u32 * 2;

    // Provide storage Nodes.
    let mut storage_nodes: Vec<TestStorage> =
        node_specs.iter().map(|spec| {
            TestStorage::new(&mut contract, spec.0, manager_id, spec.1)
        }).collect();

    assert_ne!(storage_nodes[0].node.url, storage_nodes[1].node.url, "nodes must have different URLs");

    // Provide one gateway Node.
    let gateway_node = TestGateway::new(&mut contract, accounts.alice, manager_id, "alice");

    let mut cluster_manager = ClusterManager::new(manager_id);

    // Create storage and gateway Clusters.
    cluster_manager.create_cluster(&mut contract, STORAGE_ENGINE, vnode_count);
    cluster_manager.create_cluster(&mut contract, GATEWAY_ENGINE, 1);

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut contract, accounts.bob)?;

    // Target different vnodes.
    let routing0 = (u32::MAX / vnode_count as u32) * 0 + 123;
    let routing1 = (u32::MAX / vnode_count as u32) * 1 + 123;
    let routing4 = (u32::MAX / vnode_count as u32) * 4 + 123;

    let mut execute_action = |action: Action, expect_nodes: &[usize]| {
        let request = user.make_request(&contract, action).unwrap();
        let storage_requests = gateway_node.handle_request(&contract, request).unwrap();

        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), expect_nodes.len());
        for (request_i, &node_i) in expect_nodes.iter().enumerate() {
            storage_nodes[node_i].handle_request(&contract, &storage_requests[request_i]).unwrap();
        }
    };

    // Simulate write requests to the gateway into different vnodes.
    execute_action(
        Action { routing_key: routing0, data: "data in shard 0".to_string(), op: Op::Write },
        &[0, 1, 2]);
    execute_action(
        Action { routing_key: routing1, data: "data in shard 1".to_string(), op: Op::Write },
        &[1, 2, 3]);
    execute_action(
        Action { routing_key: routing4, data: "data in shard 4".to_string(), op: Op::Write },
        &[4, 5, 0]);

    // Simulate read requests to the gateway.
    execute_action(
        Action { routing_key: routing0, data: "data in shard 0".to_string(), op: Op::Read },
        &[0, 1, 2]);
    execute_action(
        Action { routing_key: routing1, data: "data in shard 1".to_string(), op: Op::Read },
        &[1, 2, 3]);
    execute_action(
        Action { routing_key: routing4, data: "data in shard 4".to_string(), op: Op::Read },
        &[4, 5, 0]);

    // Replace a node.
    cluster_manager.replace_node(&mut contract, 0);

    let vnodes = contract.cluster_get(1).unwrap()
        .cluster.vnodes;
    assert_eq!(vnodes, vec![
        5, // Node 0 was replaced by Node 5.
        1, 2, 3, 4, 5,
        4, // Node 0 was replaced by Node 4.
        1, 2, 3, 4, 5,
    ]);

    // Check the resource distribution of all nodes.
    let (nodes, _) = contract.node_list(0, 20, None);
    let resources: Vec<Resource> = nodes.iter().map(|n| n.node.free_resource).collect();
    const INIT: u32 = 100; // Initial capacity of each node.
    const PART: u32 = 15; // Size of a vnode.
    assert_eq!(resources, vec![
        INIT, //                    Node 0 was replaced, so it got back its initial resources.
        INIT - PART * 2, //         Nodes 1,2,3 provided 2 vnodes each.
        INIT - PART * 2, //         …
        INIT - PART * 2, //         …
        INIT - PART * 2 - PART, //  Node4 provided 2 vnodes, and it took over 1 vnode from Node0.
        INIT - PART * 2 - PART, //  Node5 provided 2 vnodes, and it took over 1 vnode from Node0.
        INIT - PART, // That’s the single gateway node, not related to nodes above.
    ]);
}
