use ink_lang as ink;

use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;
use cdn_node::{entity::*};

use super::env_utils::*;
use super::setup_utils::*;


#[ink::test]
fn cdn_node_create_err_if_node_exists() {
    let mut ctx = setup_cluster();
    assert_eq!(
        ctx.contract.cdn_node_create(
            ctx.cdn_node_key1,
            ctx.cdn_node_params1,
        ),
        Err(CdnNodeAlreadyExists)
    );
}


#[ink::test]
fn cdn_node_create_success() {
    let mut ctx = setup_cluster();

    let new_provider_id = AccountId::from([0x76, 0x30, 0xc6, 0x96, 0x6f, 0xd3, 0x26, 0xba, 0x1a, 0xa0, 0x6f, 0xd8, 0x7f, 0x7b, 0xf2, 0xef, 0x14, 0x11, 0xf0, 0x0d, 0x00, 0xa9, 0xe7, 0x11, 0xdf, 0xd1, 0x65, 0x14, 0x5d, 0x01, 0xdb, 0x59]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_cdn_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);
    let new_cdn_node_params = CdnNodeParams::from("{\"url\":\"https://ddc-1.cere.network/cdn/new\"}");
    let new_cdn_node_capacity = 100;
    let new_cdn_node_rent_per_month: Balance = 10 * TOKEN;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cdn_node_create(
        new_cdn_node_key,
        new_cdn_node_params.clone()
    )?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::CdnNodeCreated(ev) if ev ==
            CdnNodeCreated {
                cdn_node_key: new_cdn_node_key,
                provider_id: new_provider_id,
                undistributed_payment: 0,
                cdn_node_params: new_cdn_node_params.clone()
            })
    );

    let cdn_node_info = ctx.contract.cdn_node_get(new_cdn_node_key)?;
    assert!(matches!(cdn_node_info.cdn_node, CdnNode {
        provider_id: new_provider_id,
        undistributed_payment: 0,
        cdn_node_params,
        cluster_id: None,
        status_in_cluster: None,
    }));

}


#[ink::test]
fn cdn_node_remove_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider_id = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_provider_id, 1000 * TOKEN);

    set_caller(not_provider_id);
    assert_eq!(
        ctx.contract.cdn_node_remove(ctx.cdn_node_key1),
        Err(OnlyCdnNodeProvider)
    );
}


#[ink::test]
fn cdn_node_remove_err_if_node_in_cluster() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id1);
    assert_eq!(
        ctx.contract.cdn_node_remove(ctx.cdn_node_key1),
        Err(CdnNodeIsAddedToCluster(ctx.cluster_id))
    );
}


#[ink::test]
fn cdn_node_remove_success() {
    let mut ctx = setup_cluster();

    set_caller(ctx.provider_id1);
    ctx.contract.cluster_remove_cdn_node(ctx.cluster_id, ctx.cdn_node_key1)?;
    ctx.contract.cdn_node_remove(ctx.cdn_node_key1)?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::CdnNodeRemoved(ev) if ev ==
            CdnNodeRemoved {
                cdn_node_key: ctx.cdn_node_key1,
            }
        )
    );

    assert_eq!(
        ctx.contract.cdn_node_get(ctx.cdn_node_key1),
        Err(CdnNodeDoesNotExist)
    );
}


#[ink::test]
fn cdn_node_set_params_err_if_not_provider() {
    let mut ctx = setup_cluster();

    let not_provider = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_provider, 1000 * TOKEN);
    // Change params.not_provider
    let new_cdn_node_params = CdnNodeParams::from("new cdn node params");
    set_caller_value(not_provider, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.cdn_node_set_params(
            ctx.cdn_node_key0, 
            new_cdn_node_params
        ),
        Err(OnlyCdnNodeProvider)
    );
}


#[ink::test]
fn node_set_params_success() {
    let mut ctx = setup_cluster();

    // Change params.
    let new_cdn_node_params = NodeParams::from("new cdn node params");
    set_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract.cdn_node_set_params(ctx.cdn_node_key0, new_cdn_node_params.clone())?;

    assert!(
        matches!(get_events().pop().unwrap(), Event::CdnNodeParamsSet(ev) if ev ==
            CdnNodeParamsSet {
                cdn_node_key: ctx.cdn_node_key0,
                cdn_node_params: new_cdn_node_params.clone()
            }
        )
    );

    // Check the changed params.
    let cdn_node_info = ctx.contract.cdn_node_get(ctx.cdn_node_key0)?;
    assert_eq!(cdn_node_info.cdn_node.cdn_node_params, new_cdn_node_params);
}

#[ink::test]
fn cdn_node_get_err_if_node_does_not_exist() {
    let ctx = setup_cluster();

    let bad_cdn_node_key = AccountId::from([0xf6, 0x8f, 0x06, 0xa8, 0x26, 0xba, 0xaf, 0x7f, 0xbd, 0x9b, 0xff, 0x3d, 0x1e, 0xec, 0xae, 0xef, 0xc7, 0x7a, 0x01, 0x6d, 0x0b, 0xaf, 0x4c, 0x90, 0x55, 0x6e, 0x7b, 0x15, 0x73, 0x46, 0x9c, 0x76]);

    assert_eq!(
        ctx.contract.cdn_node_get(bad_cdn_node_key),
        Err(CdnNodeDoesNotExist)
    );
}

#[ink::test]
fn node_get_success() {
    let ctx = setup_cluster();

    let v_nodes1_len : u32 = ctx.v_nodes1.len().try_into().unwrap();
    assert_eq!(
        ctx.contract.cdn_node_get(ctx.cdn_node_key1),
        Ok({
            CdnNodeInfo { 
                cdn_node_key: ctx.cdn_node_key1, 
                cdn_node: CdnNode {
                    provider_id: ctx.provider_id1,
                    undistributed_payment: 0,
                    cdn_node_params:ctx.cdn_node_params1,
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                }, 
            }
        })
    );
}


#[ink::test]
fn node_list_success() {
    let ctx = setup_cluster();

    let cdn_node_info = ctx.contract.cdn_node_get(ctx.cdn_node_key1)?;
    assert_eq!(ctx.provider_id1, cdn_node_info.cdn_node.provider_id.clone());

    let v_nodes1_len : u32 = ctx.v_nodes1.len().try_into().unwrap();
    let cdn_node1 = CdnNodeInfo {
        cdn_node_key: ctx.cdn_node_key1,
        cdn_node: CdnNode {
            provider_id: ctx.provider_id1,
            undistributed_payment: 0,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            cdn_node_params: ctx.cdn_node_params1.clone()
        },
    };

    let v_nodes2_len : u32 = ctx.v_nodes2.len().try_into().unwrap();
    let cdn_node2 = CdnNodeInfo {
        cdn_node_key: ctx.cdn_node_key2,
        cdn_node: CdnNode {
            provider_id: ctx.provider_id2,
            undistributed_payment: 0,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            cdn_node_params: ctx.cdn_node_params2.clone()
        }
    };

    let count = 3;

    assert_eq!(
        ctx.contract.cdn_node_list(1, 100, None),
        (vec![cdn_node1.clone(), cdn_node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.cdn_node_list(1, 2, None),
        (vec![cdn_node1.clone(), cdn_node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.cdn_node_list(1, 1, None),
        (vec![cdn_node1.clone()], count)
    );

    assert_eq!(
        ctx.contract.cdn_node_list(2, 1, None),
        (vec![cdn_node2.clone()], count)
    );

    assert_eq!(ctx.contract.cdn_node_list(21, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ctx.contract.cdn_node_list(1, 100, Some(ctx.provider_id1)),
        (vec![cdn_node1.clone()], count)
    );

    assert_eq!(
        ctx.contract.cdn_node_list(1, 100, Some(ctx.provider_id2)),
        (vec![cdn_node2.clone()], count)
    );

    let not_provider_id= AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);

    assert_eq!(
        ctx.contract.cdn_node_list(1, 100, Some(not_provider_id)),
        (vec![], count)
    );
}
