use ink_lang as ink;

use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;
use cdn_node::{entity::*};
use super::env_utils::*;
use super::setup_utils::*;


#[ink::test]
fn cluster_create_ok() {
    let ctx = setup_cluster();
    let providers_ids = &[ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];
    let node_keys = &[ctx.node_key0, ctx.node_key1, ctx.node_key2];
    let cdn_node_keys = &[ctx.cdn_node_key0, ctx.cdn_node_key1, ctx.cdn_node_key2];
    let node_params = &[ctx.node_params0.clone(), ctx.node_params1.clone(), ctx.node_params2.clone()];
    let cdn_node_params = &[ctx.cdn_node_params0.clone(), ctx.cdn_node_params1.clone(), ctx.cdn_node_params2.clone()];

    assert_eq!(ctx.cluster_id, 0, "cluster_id must start at 0");

    // Check cluster Storage nodes
    {
        let node0 = ctx.contract.node_get(ctx.node_key0)?;
        let v_nodes0 = ctx.contract.get_v_nodes_by_node(ctx.node_key0);

        assert_eq!(
            node0,
            NodeInfo {
                node_key: ctx.node_key0,
                node: Node {
                    provider_id: ctx.provider_id0,
                    rent_per_month: ctx.rent_per_month,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params0.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes0
            }
        );

        let node1 = ctx.contract.node_get(ctx.node_key1)?;
        let v_nodes1 = ctx.contract.get_v_nodes_by_node(ctx.node_key1);

        assert_eq!(
            node1,
            NodeInfo {
                node_key: ctx.node_key1,
                node: Node {
                    provider_id: ctx.provider_id1,
                    rent_per_month: ctx.rent_per_month,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params1.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes1
            }
        );

        let node2 = ctx.contract.node_get(ctx.node_key2)?;
        let v_nodes2 = ctx.contract.get_v_nodes_by_node(ctx.node_key2);

        assert_eq!(
            node2,
            NodeInfo {
                node_key: ctx.node_key2,
                node: Node {
                    provider_id: ctx.provider_id2,
                    rent_per_month: ctx.rent_per_month,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params2.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes2
            }
        );
    }

    // Check cluster CDN nodes
    {
        let cdn_node0 = ctx.contract.cdn_node_get(ctx.cdn_node_key0)?;

        assert_eq!(

            cdn_node0,
            CdnNodeInfo {
                cdn_node_key: ctx.cdn_node_key0,
                cdn_node: CdnNode {
                    provider_id: ctx.provider_id0,
                    undistributed_payment: 0,
                    cdn_node_params: ctx.cdn_node_params0.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
            }
        );

        let cdn_node1 = ctx.contract.cdn_node_get(ctx.cdn_node_key1)?;

        assert_eq!(
            cdn_node1,
            CdnNodeInfo {
                cdn_node_key: ctx.cdn_node_key1,
                cdn_node: CdnNode {
                    provider_id: ctx.provider_id1,
                    undistributed_payment: 0,
                    cdn_node_params: ctx.cdn_node_params1.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
            }
        );

        let cdn_node2 = ctx.contract.cdn_node_get(ctx.cdn_node_key2)?;

        assert_eq!(
            cdn_node2,
            CdnNodeInfo {
                cdn_node_key: ctx.cdn_node_key2,
                cdn_node: CdnNode {
                    provider_id: ctx.provider_id2,
                    undistributed_payment: 0,
                    cdn_node_params: ctx.cdn_node_params2.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
            }
        );
    }

    // Check the cluster
    {
        let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
        let cluster_v_nodes = ctx.contract.get_v_nodes_by_cluster(ctx.cluster_id);
        assert_eq!(
            cluster,
            ClusterInfo {
                cluster_id: ctx.cluster_id,
                cluster: Cluster {
                    manager_id: ctx.manager_id,
                    nodes_keys: ctx.nodes_keys,
                    resource_per_v_node: ctx.reserved_resource,
                    resource_used: 0,
                    cluster_params: ctx.cluster_params.clone(),
                    revenues: Cash(0),
                    total_rent: ctx.rent_per_month * ctx.cluster_v_nodes.len() as Balance,
                    cdn_nodes_keys: ctx.cdn_nodes_keys,
                    cdn_usd_per_gb: CDN_USD_PER_GB,
                    cdn_revenues: Cash(0),
                },
                cluster_v_nodes
            }
        );
    }

    // Check emitted events
    let mut events = get_events();
    events.reverse(); // Work with pop().

    // Storage node created event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::NodeCreated(ev) if ev ==
            NodeCreated {
                node_key: node_keys[i],
                provider_id: providers_ids[i],
                rent_per_month: ctx.rent_per_month,
                node_params: node_params[i].clone()
            })
        );
    }

    // CDN node created event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::CdnNodeCreated(ev) if ev ==
            CdnNodeCreated {
                cdn_node_key: cdn_node_keys[i],
                provider_id: providers_ids[i],
                cdn_node_params: cdn_node_params[i].clone(),
                undistributed_payment: 0
            })
        );
    }

    // Cluster created event
    assert!(
        matches!(events.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { 
            cluster_id: ctx.cluster_id, 
            manager: ctx.manager_id, 
            cluster_params: ctx.cluster_params.clone()
        })
    );

    // Permission granted event
    for provider_id in providers_ids {
        assert!(
            matches!(events.pop().unwrap(), Event::PermissionGranted(ev) if ev ==
            PermissionGranted { 
                account_id: ctx.manager_id, 
                permission: Permission::ClusterManagerTrustedBy(*provider_id) 
            })
        );
    }

    // Cluster storage node added event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::ClusterNodeAdded(ev) if ev ==
            ClusterNodeAdded {
                cluster_id: ctx.cluster_id,
                node_key: node_keys[i]
            })
        );
    }

    // Cluster cdn node added event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::ClusterCdnNodeAdded(ev) if ev ==
            ClusterCdnNodeAdded {
                cluster_id: ctx.cluster_id,
                cdn_node_key: cdn_node_keys[i]
            })
        );
    }

    // Cluster resource reserved event
    assert!(
        matches!(events.pop().unwrap(), Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { 
            cluster_id: ctx.cluster_id, 
            resource: ctx.reserved_resource 
        })
    );

    assert_eq!(events.len(), 0, "All events must be checked");

}


#[ink::test]
fn cluster_add_node_err_if_node_is_in_cluster() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(ClusterParams::from("{}"))?;

    assert_eq!(
        ctx.contract.cluster_add_node(
            another_cluster_id,
            ctx.node_key1, 
            ctx.v_nodes1, 
        ),
        Err(NodeIsAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn cluster_add_node_err_if_not_trusted_manager() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(
        ClusterParams::from("{}")
    )?;

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    let new_node_key = ctx.contract.node_create(
        new_node_key,
        NodeParams::from("new_node"),
        1000,
        100
    )?;

    let new_v_nodes: Vec<VNodeToken> = vec![10, 11, 12];
    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    assert_eq!(
        ctx.contract.cluster_add_node(
            another_cluster_id,
            new_node_key, 
            new_v_nodes, 
        ),
        Err(OnlyTrustedClusterManager)
    );
}


#[ink::test]
fn cluster_add_node_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(
        ClusterParams::from("{}")
    )?;

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.node_create(
        new_node_key,
        NodeParams::from("new_node"),
        1000,
        100
    )?;

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.grant_trusted_manager_permission(not_manager_id)?;

    let new_v_nodes: Vec<VNodeToken> = vec![10, 11, 12];
    set_caller_value(not_manager_id, CONTRACT_FEE_LIMIT);
    assert_eq!(
        ctx.contract.cluster_add_node(
            another_cluster_id,
            new_node_key, 
            new_v_nodes, 
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_add_node_ok() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);
    let new_node_rent_per_month = 100;
    let new_node_params = NodeParams::from("new_node");
    let new_node_capacity = 1000;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.node_create(
        new_node_key,
        new_node_params.clone(),
        new_node_capacity,
        new_node_rent_per_month
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::NodeCreated(ev) if ev ==
            NodeCreated {
                node_key: new_node_key,
                provider_id: new_provider_id,
                rent_per_month: new_node_rent_per_month,
                node_params: new_node_params
            })
    );

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.grant_trusted_manager_permission(ctx.manager_id)?;

    assert!(
        matches!(
            get_events().pop().unwrap(), Event::PermissionGranted(ev) if ev == 
            PermissionGranted { 
                account_id: ctx.manager_id, 
                permission: Permission::ClusterManagerTrustedBy(new_provider_id) 
            }
        )
    );

    let new_v_nodes: Vec<VNodeToken> = vec![10, 11, 12];
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_add_node(
        ctx.cluster_id, 
        new_node_key, 
        new_v_nodes.clone()
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterNodeAdded(ev) if ev ==
        ClusterNodeAdded {
            cluster_id: ctx.cluster_id,
            node_key: new_node_key
        })
    );

    let nodes_keys = vec![
        ctx.node_key0,
        ctx.node_key1,
        ctx.node_key2,
        new_node_key,
    ];

    let cluster_v_nodes = vec![
        ctx.v_nodes0,
        ctx.v_nodes1,
        ctx.v_nodes2,
        new_v_nodes
    ];

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert!(matches!(cluster_info.cluster.nodes_keys, nodes_keys));
    assert!(matches!(cluster_info.cluster_v_nodes, cluster_v_nodes));
}


#[ink::test]
fn cluster_remove_node_err_if_node_is_not_in_cluster() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let another_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.node_create(
        another_node_key,
        NodeParams::from("new_node"),
        1000,
        100
    )?;

    set_caller(ctx.manager_id);
    assert_eq!(
        ctx.contract.cluster_remove_node(
            ctx.cluster_id,
            another_node_key, 
        ),
        Err(NodeIsNotAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn cluster_remove_node_err_if_not_manager_and_not_provider() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller(not_manager_id);
    assert_eq!(
        ctx.contract.cluster_remove_node(
            ctx.cluster_id,
            ctx.node_key1, 
        ),
        Err(OnlyClusterManagerOrNodeProvider)
    );
}


#[ink::test]
fn cluster_remove_node_ok_if_node_provider() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    ctx.contract.cluster_remove_node(
        ctx.cluster_id,
        ctx.node_key1
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterNodeRemoved(ev) if ev ==
        ClusterNodeRemoved {
            cluster_id: ctx.cluster_id,
            node_key: ctx.node_key1
        })
    );

    let nodes_keys = vec![
        ctx.node_key0,
        ctx.node_key2,
    ];

    let cluster_v_nodes = vec![
        ctx.v_nodes0,
        ctx.v_nodes2,
    ];

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert!(matches!(cluster_info.cluster.nodes_keys, nodes_keys));
    assert!(matches!(cluster_info.cluster_v_nodes, cluster_v_nodes));
    
}


#[ink::test]
fn cluster_remove_node_ok_if_cluster_manager() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id2);
    ctx.contract.cluster_remove_node(
        ctx.cluster_id,
        ctx.node_key2
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterNodeRemoved(ev) if ev ==
        ClusterNodeRemoved {
            cluster_id: ctx.cluster_id,
            node_key: ctx.node_key2
        })
    );

    let nodes_keys = vec![
        ctx.node_key0,
        ctx.node_key1,
    ];

    let cluster_v_nodes = vec![
        ctx.v_nodes0,
        ctx.v_nodes1,
    ];

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert!(matches!(cluster_info.cluster.nodes_keys, nodes_keys));
    assert!(matches!(cluster_info.cluster_v_nodes, cluster_v_nodes));
    
}


#[ink::test]
fn cluster_add_cdn_node_err_if_cdn_node_is_in_cluster() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(ClusterParams::from("{}"))?;

    assert_eq!(
        ctx.contract.cluster_add_cdn_node(
            another_cluster_id,
            ctx.cdn_node_key1, 
        ),
        Err(CdnNodeIsAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn cluster_add_cdn_node_err_if_not_trusted_manager() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(ClusterParams::from("{}"))?;

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_cdn_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    let new_node_key = ctx.contract.cdn_node_create(
        new_cdn_node_key,
        CdnNodeParams::from("new_cdn_node"),
    )?;

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    assert_eq!(
        ctx.contract.cluster_add_cdn_node(
            another_cluster_id,
            new_cdn_node_key, 
        ),
        Err(OnlyTrustedClusterManager)
    );
}


#[ink::test]
fn cluster_add_cdn_node_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let another_manager_id = AccountId::from([0x54, 0x66, 0x76, 0x6c, 0xf6, 0x17, 0x70, 0xcf, 0x5d, 0x70, 0x6c, 0x55, 0x4d, 0xd4, 0xb7, 0xf8, 0x83, 0xe6, 0x70, 0x06, 0xea, 0x4c, 0x05, 0x89, 0x16, 0x32, 0x79, 0x79, 0xbb, 0x85, 0x58, 0x7a]);
    set_balance(another_manager_id, 1000 * TOKEN);

    set_caller_value(another_manager_id, CONTRACT_FEE_LIMIT);
    let another_cluster_id = ctx.contract.cluster_create(ClusterParams::from("{}"))?;

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_cdn_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cdn_node_create(
        new_cdn_node_key,
        CdnNodeParams::from("new_cdn_node"),
    )?;

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.grant_trusted_manager_permission(not_manager_id)?;

    set_caller_value(not_manager_id, CONTRACT_FEE_LIMIT);
    assert_eq!(
        ctx.contract.cluster_add_cdn_node(
            another_cluster_id,
            new_cdn_node_key, 
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_add_cdn_node_ok() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_cdn_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);
    let new_cdn_node_params = CdnNodeParams::from("new_cdn_node");

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cdn_node_create(
        new_cdn_node_key,
        new_cdn_node_params.clone(),
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::CdnNodeCreated(ev) if ev ==
            CdnNodeCreated {
                cdn_node_key: new_cdn_node_key,
                provider_id: new_provider_id,
                cdn_node_params: new_cdn_node_params,
                undistributed_payment: 0
            })
    );

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.grant_trusted_manager_permission(ctx.manager_id)?;

    assert!(
        matches!(
            get_events().pop().unwrap(), Event::PermissionGranted(ev) if ev == 
            PermissionGranted { 
                account_id: ctx.manager_id, 
                permission: Permission::ClusterManagerTrustedBy(new_provider_id) 
            }
        )
    );

    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_add_cdn_node(
        ctx.cluster_id, 
        new_cdn_node_key, 
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterCdnNodeAdded(ev) if ev ==
        ClusterCdnNodeAdded {
            cluster_id: ctx.cluster_id,
            cdn_node_key: new_cdn_node_key
        })
    );

    let cdn_nodes_keys = vec![
        ctx.cdn_node_key0,
        ctx.cdn_node_key1,
        ctx.cdn_node_key2,
        new_cdn_node_key,
    ];

    let cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert!(matches!(cluster_info.cluster.cdn_nodes_keys, cdn_nodes_keys));
}


#[ink::test]
fn cluster_remove_cdn_node_err_if_cdn_node_is_not_in_cluster() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let another_cdn_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cdn_node_create(
        another_cdn_node_key,
        CdnNodeParams::from("new_cdn_node")
    )?;

    set_caller(ctx.manager_id);
    assert_eq!(
        ctx.contract.cluster_remove_cdn_node(
            ctx.cluster_id,
            another_cdn_node_key, 
        ),
        Err(CdnNodeIsNotAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn cluster_remove_cdn_node_err_if_not_manager_and_not_provider() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller(not_manager_id);
    assert_eq!(
        ctx.contract.cluster_remove_cdn_node(
            ctx.cluster_id,
            ctx.cdn_node_key1, 
        ),
        Err(OnlyClusterManagerOrCdnNodeProvider)
    );
}


#[ink::test]
fn cluster_remove_cdn_node_ok_if_cdn_node_provider() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    ctx.contract.cluster_remove_cdn_node(
        ctx.cluster_id,
        ctx.cdn_node_key1
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterCdnNodeRemoved(ev) if ev ==
        ClusterCdnNodeRemoved {
            cluster_id: ctx.cluster_id,
            cdn_node_key: ctx.cdn_node_key1
        })
    );

    let cdn_nodes_keys = vec![
        ctx.cdn_node_key0,
        ctx.cdn_node_key2,
    ];

    let cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert!(matches!(cluster_info.cluster.cdn_nodes_keys, cdn_nodes_keys));
    
}


#[ink::test]
fn cluster_remove_cdn_node_ok_if_cluster_manager() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id2);
    ctx.contract.cluster_remove_cdn_node(
        ctx.cluster_id,
        ctx.cdn_node_key2
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterCdnNodeRemoved(ev) if ev ==
        ClusterCdnNodeRemoved {
            cluster_id: ctx.cluster_id,
            cdn_node_key: ctx.cdn_node_key2
        })
    );

    let cdn_nodes_keys = vec![
        ctx.cdn_node_key0,
        ctx.cdn_node_key1,
    ];

    let cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert!(matches!(cluster_info.cluster.cdn_nodes_keys, cdn_nodes_keys));
    
}


#[ink::test]
fn cluster_set_params_err_if_not_cluster_manager() {
    let ctx = &mut setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);
    // Change params.
    let new_cluster_params = NodeParams::from("new cluster params");
    set_caller_value(not_manager_id, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.cluster_set_params(
            ctx.cluster_id, 
            new_cluster_params
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_set_params_ok() {
    let mut ctx = setup_cluster();

    // Change params.
    let new_cluster_params = NodeParams::from("new cluster params");
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_set_params(ctx.cluster_id, new_cluster_params.clone())?;

    // Check the changed params.
    let cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(cluster_info.cluster.cluster_params, new_cluster_params);
}


#[ink::test]
fn cluster_replace_node_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_caller_value(not_manager_id, 0);

    // Reassign a vnode from node1 to node2.
    assert_eq!(
        ctx.contract.cluster_replace_node(
            ctx.cluster_id, 
            vec![1, 2, 3], 
            ctx.node_key2
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_replace_node_err_if_node_does_not_exist() {
    let mut ctx = setup_cluster();

    let bad_node_key = AccountId::from([0xf6, 0x8f, 0x06, 0xa8, 0x26, 0xba, 0xaf, 0x7f, 0xbd, 0x9b, 0xff, 0x3d, 0x1e, 0xec, 0xae, 0xef, 0xc7, 0x7a, 0x01, 0x6d, 0x0b, 0xaf, 0x4c, 0x90, 0x55, 0x6e, 0x7b, 0x15, 0x73, 0x46, 0x9c, 0x76]);
    set_caller(ctx.manager_id);
    assert_eq!(
        ctx.contract.cluster_replace_node(
            ctx.cluster_id, 
            vec![1, 2, 3],
            bad_node_key
        ),
        Err(NodeDoesNotExist)
    );
}


#[ink::test]
fn cluster_replace_node_ok() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    // Reassign a vnode from node0 to node2.
    ctx.contract.cluster_replace_node(
        ctx.cluster_id, 
        vec![1, 3], 
        ctx.node_key2
    )?;

    // Check the last event
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterNodeReplaced(ev) if ev ==
        ClusterNodeReplaced { 
            cluster_id: ctx.cluster_id, 
            node_key: ctx.node_key2
        }
    ));

    let mut cluster_v_nodes = Vec::<VNodeToken>::new();
    cluster_v_nodes.extend(vec![2]);
    cluster_v_nodes.extend(ctx.v_nodes1.clone());
    cluster_v_nodes.extend(ctx.v_nodes2.clone());
    cluster_v_nodes.extend(vec![1, 3]);
    cluster_v_nodes.sort();

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert_eq!(&cluster_info.cluster_v_nodes, &cluster_v_nodes, "a v_node must be replaced");

    let mut v_nodes0 = ctx.contract.get_v_nodes_by_node(ctx.node_key0.clone());
    v_nodes0.sort();
    let mut v_nodes1 = ctx.contract.get_v_nodes_by_node(ctx.node_key1.clone());
    v_nodes1.sort();
    let mut v_nodes2 = ctx.contract.get_v_nodes_by_node(ctx.node_key2.clone());
    v_nodes2.sort();

    assert_eq!(&v_nodes0, &vec![2], "v_nodes must be replaced for the 1st node");
    assert_eq!(&v_nodes1, &vec![4, 5, 6], "v_nodes must not be replaced for the 2nd node");
    assert_eq!(&v_nodes2, &vec![1, 3, 7, 8, 9], "v_nodes must be assigned to the 3rd node");

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_key0, 100 - 10),
        (ctx.node_key1, 100 - 10 - 10 - 10),
        (ctx.node_key2, 100 - 10 - 10 - 10 - 10 - 10),
    ];

    for (node_key, available) in expected_resources {
        let node_info = ctx.contract.node_get(node_key).unwrap();
        assert_eq!(
            node_info.node.free_resource, available,
            "resources must have shifted between nodes"
        );
    }
}


#[ink::test]
fn cluster_reserve_resource_ok() {
    let mut ctx = setup_cluster();
    set_caller(ctx.manager_id);

    // Reserve more resources.
    ctx.contract.cluster_reserve_resource(ctx.cluster_id, 5);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { 
            cluster_id: ctx.cluster_id, 
            resource: 5 
        }
    ));

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    assert_eq!(cluster.resource_per_v_node, 10 + 5);

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_key0, 100 - 40 - 5),
        (ctx.node_key1, 100 - 40 - 5),
        (ctx.node_key2, 100 - 40 - 5),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.node.free_resource,
            available,
            "more resources must be reserved from the nodes"
        );
    }
}


#[ink::test]
fn cluster_distribute_revenue_ok() {
    let ctx = &mut setup_cluster();
    let test_bucket = &setup_bucket(ctx);
    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>();
    // Pay the due thus far.
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_settle_payment(test_bucket.bucket_id);

    // Get state before the distribution.
    let to_distribute = ctx
        .contract
        .cluster_get(ctx.cluster_id)?
        .cluster
        .revenues
        .peek();

    let before0 = balance_of(ctx.provider_id0);
    let before1 = balance_of(ctx.provider_id1);
    let before2 = balance_of(ctx.provider_id2);
    let before_mgmt = balance_of(ctx.manager_id);

    let skip_events = get_events::<Event>().len();

    // Set a network fee.
    let network_fee_bp = 100; // 1%
    let cluster_management_fee_bp = 200; // 2%
    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    ctx.contract.admin_set_fee_config(FeeConfig {
        network_fee_bp,
        network_fee_destination: AccountId::default(),
        cluster_management_fee_bp,
    });

    let burned_fee = to_distribute * network_fee_bp / 10_000;
    let manager_fee = (to_distribute - burned_fee) * cluster_management_fee_bp / 10_000;
    let provider_fee: u128 = (to_distribute - burned_fee - manager_fee) / 3;

    // Distribute the revenues of the cluster to providers.
    ctx.contract.cluster_distribute_revenues(ctx.cluster_id);

    // Check the last events.
    let mut events = get_events();
    events.reverse(); // Work with pop().
    events.truncate(events.len() - skip_events);
    let expected_recipients = vec![ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];

    for provider_id in expected_recipients {
        assert!(
            matches!(events.pop().unwrap(), Event::ClusterDistributeRevenues(ev) if ev ==
            ClusterDistributeRevenues { 
                cluster_id: ctx.cluster_id, 
                provider_id 
            })
        );
    }

    assert_eq!(events.len(), 0, "all events must be checked");

    // Get state after the distribution.
    let rounding_error = ctx
        .contract
        .cluster_get(ctx.cluster_id)?
        .cluster
        .revenues
        .peek();

    let earned0 = balance_of(ctx.provider_id0) - before0;
    let earned1 = balance_of(ctx.provider_id1) - before1;
    let earned2 = balance_of(ctx.provider_id2) - before2;
    let earned_mgmt = balance_of(ctx.manager_id) - before_mgmt;

    assert!(provider_fee > 0, "provider must earn something");
    assert_eq!(
        earned0, provider_fee,
        "providers must earn the correct amount"
    );
    assert_eq!(
        earned1, provider_fee,
        "providers must earn the correct amount"
    );
    assert_eq!(
        earned2, provider_fee,
        "providers must earn the correct amount"
    );

    assert!(burned_fee > 0, "the network must earn something");
    assert!(manager_fee > 0, "the manager_id must earn something");
    assert_eq!(
        earned_mgmt, manager_fee,
        "the manager_id must earn the correct amount"
    );

    assert!(to_distribute > 0);
    assert!(
        rounding_error < 10,
        "revenues must go out of the cluster (besides rounding)"
    );
    assert_eq!(
        earned0 + earned1 + earned2 + burned_fee + manager_fee + rounding_error,
        to_distribute,
        "all revenues must go to providers"
    );
}


#[ink::test]
fn cluster_remove_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller(not_manager_id);
    assert_eq!(
        ctx.contract.cluster_remove(ctx.cluster_id),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_remove_err_if_cluster_is_not_empty() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    assert_eq!(
        ctx.contract.cluster_remove(ctx.cluster_id),
        Err(ClusterIsNotEmpty)
    );
}


#[ink::test]
fn cluster_remove_ok() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);

    ctx.contract.cluster_remove_node(
        ctx.cluster_id,
        ctx.node_key0, 
    )?;

    ctx.contract.cluster_remove_node(
        ctx.cluster_id,
        ctx.node_key1, 
    )?;

    ctx.contract.cluster_remove_node(
        ctx.cluster_id,
        ctx.node_key2, 
    )?;

    ctx.contract.cluster_remove_cdn_node(
        ctx.cluster_id,
        ctx.cdn_node_key0, 
    )?;

    ctx.contract.cluster_remove_cdn_node(
        ctx.cluster_id,
        ctx.cdn_node_key1, 
    )?;

    ctx.contract.cluster_remove_cdn_node(
        ctx.cluster_id,
        ctx.cdn_node_key2, 
    )?;

    ctx.contract.cluster_remove(ctx.cluster_id)?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::ClusterRemoved(ev) if ev ==
        ClusterRemoved {
            cluster_id: ctx.cluster_id,
        })
    );

    assert_eq!(
        ctx.contract.cluster_get(ctx.cluster_id),
        Err(ClusterDoesNotExist)
    );

}


#[ink::test]
fn cluster_set_node_status_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller(not_manager_id);
    assert_eq!(
        ctx.contract.cluster_set_node_status(
            ctx.cluster_id,
            ctx.node_key0,
            NodeStatusInCluster::ACTIVE
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_set_node_status_ok() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    ctx.contract.cluster_set_node_status(
        ctx.cluster_id,
        ctx.node_key0,
        NodeStatusInCluster::ACTIVE
    )?;

    let node_info = ctx.contract.node_get(ctx.node_key0)?;
    assert_eq!(
        node_info.node.status_in_cluster,
        Some(NodeStatusInCluster::ACTIVE)
    );
}


#[ink::test]
fn cluster_set_cdn_node_status_err_if_not_cluster_manager() {
    let mut ctx = setup_cluster();

    let not_manager_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager_id, 1000 * TOKEN);

    set_caller(not_manager_id);
    assert_eq!(
        ctx.contract.cluster_set_cdn_node_status(
            ctx.cluster_id,
            ctx.cdn_node_key0,
            NodeStatusInCluster::ACTIVE
        ),
        Err(OnlyClusterManager)
    );
}


#[ink::test]
fn cluster_set_cdn_node_status_ok() {
    let mut ctx = setup_cluster();

    set_caller(ctx.manager_id);
    ctx.contract.cluster_set_cdn_node_status(
        ctx.cluster_id,
        ctx.cdn_node_key0,
        NodeStatusInCluster::ACTIVE
    )?;

    let cdn_node_info = ctx.contract.cdn_node_get(ctx.cdn_node_key0)?;
    assert_eq!(
        cdn_node_info.cdn_node.status_in_cluster,
        Some(NodeStatusInCluster::ACTIVE)
    );
}


#[ink::test]
fn cluster_distribute_cdn_revenue_ok() {
    // todo: this test scenario must be revised as it does pure printing without any assertion
    println!("Creating new cdn cluster");

    let mut ctx = setup_cluster();

    // The provider stops trusting the manager_id.
    println!("Cluster id is {}", ctx.cluster_id);
    set_caller(ctx.provider_id0);

    let usd_per_cere = TOKEN / 100;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere);

    let usd_amount = ctx.contract.account_get_usd_per_cere();
    println!("Current usd amount is {}", usd_amount);

    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);
    println!("The current rate is {}", rate);

    let usd_per_kb = rate / KB_PER_GB;
    println!("The current rate per kb {}", usd_per_kb);

    let cere_per_kb = ctx.contract.accounts.1.to_cere(usd_per_kb);
    println!("The current cere rate per kb {}", cere_per_kb);

    set_caller_value(ctx.provider_id0, 10 * TOKEN);
    ctx.contract.account_deposit();

    set_caller_value(ctx.provider_id0, 10 * TOKEN);
    ctx.contract.account_bond(5 * TOKEN);

    set_caller(ctx.provider_id0);
    ctx.contract.set_fee_bp(1_000);

    let account0_before_putting = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("Before putting revenue: {:?}", account0_before_putting);

    ctx.contract.cluster_put_cdn_revenue(
        ctx.cluster_id,
        vec![(ctx.provider_id0, 1000), (ctx.provider_id0, 541643)],
        vec![(ctx.cdn_node_key0, 1000), (ctx.cdn_node_key1, 541643)],
        vec![],
        5,
    );
    let account0_after_putting = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("After putting revenue: {:?}", account0_after_putting);

    let cluster_list_1 = ctx.contract.cluster_list(0, 10, None);
    println!("Cluster list one {:?}", cluster_list_1);
    let cdn_node0 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key0).unwrap();
    let cdn_node1 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key1).unwrap();

    println!("{:?}", cdn_node0);
    println!("{:?}", cdn_node1);
    let cluster0 = ctx.contract.clusters.get(ctx.cluster_id);
    println!("{:?}", cluster0);
    let cluster_list = ctx.contract.cluster_list(0, 10, None);
    println!("{:?}", cluster0);
    println!("{:?}", cluster_list);

    let validated_commit_node0 = ctx.contract.get_validated_commit(ctx.cdn_node_key0);
    println!("Validated commit: {:?}", validated_commit_node0);

    let fee = ctx.contract.get_fee_bp();
    println!("Protocol fee in basis points: {:?}", fee);

    let protocol_revenues = ctx.contract.get_protocol_revenues();
    println!("Protocol revenues: {:?}", protocol_revenues);

    set_caller(ctx.provider_id0);

    ctx.contract.cluster_distribute_cdn_revenue(ctx.cluster_id);

    let cdn_node0 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key0).unwrap();
    let cdn_node1 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key1).unwrap();
    println!("{:?}", cdn_node0);
    println!("{:?}", cdn_node1);

    let cluster_list_1 = ctx.contract.cluster_list(0, 10, None);
    println!("{:?}", cluster_list_1);

    let account0_after_distributing = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("{:?}", account0_after_distributing);

}
