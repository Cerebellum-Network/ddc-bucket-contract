use ink_lang as ink;

use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;
use super::env_utils::*;
use super::setup_utils::*;


#[ink::test]
fn node_remove_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_provider, 1000 * TOKEN);

    set_caller(not_provider);
    assert_eq!(
        ctx.contract.node_remove(
            ctx.node_key1,
        ),
        Err(OnlyNodeProvider)
    );
}


#[ink::test]
fn node_remove_err_if_node_in_cluster() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id1);
    assert_eq!(
        ctx.contract.node_remove(
            ctx.node_key1,
        ),
        Err(NodeIsAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn node_set_params_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_provider, 1000 * TOKEN);
    // Change params.not_provider
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(not_provider, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.node_set_params(
            ctx.node_key0, 
            new_node_params
        ),
        Err(OnlyNodeProvider)
    );
}


#[ink::test]
fn node_set_params_success() {
    let mut ctx = setup_cluster();

    // Change params.
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract.node_set_params(ctx.node_key0, new_node_params.clone())?;

    // Check the changed params.
    let status = ctx.contract.node_get(ctx.node_key0)?;
    assert_eq!(status.node.node_params, new_node_params);
}


#[ink::test]
fn node_list_success() {
    let ctx = setup_cluster();

    let node_info = ctx.contract.node_get(ctx.node_key1)?;
    assert_eq!(ctx.provider_id1, node_info.node.provider_id.clone());

    let node1 = NodeInfo {
        node_key: ctx.node_key1,
        node: Node {
            provider_id: ctx.provider_id1,
            rent_per_month: ctx.rent_per_v_node,
            free_resource: ctx.capacity - ctx.reserved_resource * 3,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params1,
        },
        v_nodes: ctx.v_nodes1.clone()
    };

    let node2 = NodeInfo {
        node_key: ctx.node_key2,
        node: Node {
            provider_id:ctx.provider_id2,
            rent_per_month: ctx.rent_per_v_node,
            free_resource: ctx.capacity - ctx.reserved_resource * 3,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params2,
        },
        v_nodes: ctx.v_nodes2.clone()
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
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ctx.contract.node_list(2, 1, None),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    assert_eq!(ctx.contract.node_list(21, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id1)),
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id2)),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    let not_provider= AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(not_provider)),
        (vec![], count)
    );
}
