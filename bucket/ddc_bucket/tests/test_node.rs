use ink_lang as ink;

use super::env_utils::*;
use super::setup_utils::*;
use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;

#[ink::test]
fn node_create_err_if_node_exists() {
    let mut ctx = setup_cluster();
    assert_eq!(
        ctx.contract.node_create(
            ctx.node_key1,
            ctx.node_params1,
            ctx.node_capacity1,
            ctx.rent_v_node_per_month1,
        ),
        Err(NodeAlreadyExists)
    );
}

#[ink::test]
fn node_create_ok() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([
        0x76, 0x30, 0xc6, 0x96, 0x6f, 0xd3, 0x26, 0xba, 0x1a, 0xa0, 0x6f, 0xd8, 0x7f, 0x7b, 0xf2,
        0xef, 0x14, 0x11, 0xf0, 0x0d, 0x00, 0xa9, 0xe7, 0x11, 0xdf, 0xd1, 0x65, 0x14, 0x5d, 0x01,
        0xdb, 0x59,
    ]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    let new_node_params = NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/new\"}");
    let new_node_capacity = 100;
    let new_node_rent_v_node_per_month: Balance = 10 * TOKEN;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.node_create(
        new_node_key,
        new_node_params.clone(),
        new_node_capacity,
        new_node_rent_v_node_per_month,
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::NodeCreated(ev) if ev ==
        NodeCreated {
            node_key: new_node_key,
            provider_id: new_provider_id,
            rent_v_node_per_month: new_node_rent_v_node_per_month,
            node_params: new_node_params.clone()
        })
    );

    let node_info = ctx.contract.node_get(new_node_key)?;
    let _expected_node_info = Node {
        provider_id: new_provider_id,
        rent_v_node_per_month: new_node_rent_v_node_per_month,
        free_resource: new_node_capacity,
        node_params: new_node_params,
        cluster_id: None,
        status_in_cluster: None,
    };
    assert!(matches!(node_info.node, _expected_node_info));
}

#[ink::test]
fn node_remove_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider_id = AccountId::from([
        0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6,
        0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4,
        0xf8, 0x6d,
    ]);
    set_balance(not_provider_id, 1000 * TOKEN);

    set_caller(not_provider_id);
    assert_eq!(
        ctx.contract.node_remove(ctx.node_key1),
        Err(OnlyNodeProvider)
    );
}

#[ink::test]
fn node_remove_err_if_node_in_cluster() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id1);
    assert_eq!(
        ctx.contract.node_remove(ctx.node_key1),
        Err(NodeIsAddedToCluster(ctx.cluster_id))
    );
}

#[ink::test]
fn node_remove_ok() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id1);
    ctx.contract
        .cluster_remove_node(ctx.cluster_id, ctx.node_key1)?;
    ctx.contract.node_remove(ctx.node_key1)?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::NodeRemoved(ev) if ev ==
            NodeRemoved {
                node_key: ctx.node_key1,
            }
        )
    );

    assert_eq!(ctx.contract.node_get(ctx.node_key1), Err(NodeDoesNotExist));
}

#[ink::test]
fn node_set_params_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider = AccountId::from([
        0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6,
        0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4,
        0xf8, 0x6d,
    ]);
    set_balance(not_provider, 1000 * TOKEN);
    // Change params.not_provider
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(not_provider, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.node_set_params(ctx.node_key0, new_node_params),
        Err(OnlyNodeProvider)
    );
}

#[ink::test]
fn node_set_params_ok() {
    let mut ctx = setup_cluster();

    // Change params.
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract
        .node_set_params(ctx.node_key0, new_node_params.clone())?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::NodeParamsSet(ev) if ev ==
            NodeParamsSet {
                node_key: ctx.node_key0,
                node_params: new_node_params.clone()
            }
        )
    );

    // Check the changed params.
    let node_info = ctx.contract.node_get(ctx.node_key0)?;
    assert_eq!(node_info.node.node_params, new_node_params);
}

#[ink::test]
fn node_get_err_if_node_does_not_exist() {
    let ctx = setup_cluster();

    let bad_node_key = AccountId::from([
        0xf6, 0x8f, 0x06, 0xa8, 0x26, 0xba, 0xaf, 0x7f, 0xbd, 0x9b, 0xff, 0x3d, 0x1e, 0xec, 0xae,
        0xef, 0xc7, 0x7a, 0x01, 0x6d, 0x0b, 0xaf, 0x4c, 0x90, 0x55, 0x6e, 0x7b, 0x15, 0x73, 0x46,
        0x9c, 0x76,
    ]);

    assert_eq!(ctx.contract.node_get(bad_node_key), Err(NodeDoesNotExist));
}

#[ink::test]
fn node_get_ok() {
    let ctx = setup_cluster();

    let v_nodes1_len: u32 = ctx.v_nodes1.len().try_into().unwrap();
    assert_eq!(
        ctx.contract.node_get(ctx.node_key1),
        Ok({
            NodeInfo {
                node_key: ctx.node_key1,
                node: Node {
                    provider_id: ctx.provider_id1,
                    rent_v_node_per_month: ctx.rent_v_node_per_month1,
                    free_resource: ctx.node_capacity1 - ctx.resource_per_v_node * v_nodes1_len,
                    node_params: ctx.node_params1,
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: ctx.v_nodes1,
            }
        })
    );
}

#[ink::test]
fn node_list_ok() {
    let ctx = setup_cluster();

    let node_info = ctx.contract.node_get(ctx.node_key1)?;
    assert_eq!(ctx.provider_id1, node_info.node.provider_id.clone());

    let v_nodes1_len: u32 = ctx.v_nodes1.len().try_into().unwrap();
    let node1 = NodeInfo {
        node_key: ctx.node_key1,
        node: Node {
            provider_id: ctx.provider_id1,
            rent_v_node_per_month: ctx.rent_v_node_per_month1,
            free_resource: ctx.node_capacity1 - ctx.resource_per_v_node * v_nodes1_len,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params1,
        },
        v_nodes: ctx.v_nodes1.clone(),
    };

    let v_nodes2_len: u32 = ctx.v_nodes2.len().try_into().unwrap();
    let node2 = NodeInfo {
        node_key: ctx.node_key2,
        node: Node {
            provider_id: ctx.provider_id2,
            rent_v_node_per_month: ctx.rent_v_node_per_month2,
            free_resource: ctx.node_capacity2 - ctx.resource_per_v_node * v_nodes2_len,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params2,
        },
        v_nodes: ctx.v_nodes2.clone(),
    };

    let count = 3;

    assert_eq!(
        ctx.contract.node_list(1, 100, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 2, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 1, None),
        (vec![node1.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(2, 1, None),
        (vec![node2.clone()], count)
    );

    assert_eq!(ctx.contract.node_list(21, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id1)),
        (vec![node1.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id2)),
        (vec![node2.clone()], count)
    );

    let not_provider_id = AccountId::from([
        0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6,
        0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4,
        0xf8, 0x6d,
    ]);

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(not_provider_id)),
        (vec![], count)
    );
}
