
use crate::ddc_bucket::*;
use super::env_utils::*;


pub fn setup_contract() -> DdcBucket {
    set_caller(admin_id());
    set_callee(contract_id());
    let contract = DdcBucket::new();
    set_balance(contract_id(), 10);
    contract
}


pub struct TestCluster {
    pub contract: DdcBucket,

    pub provider_id0: AccountId,
    pub provider_id1: AccountId,
    pub provider_id2: AccountId,

    pub node_key0: NodeKey,
    pub node_key1: NodeKey,
    pub node_key2: NodeKey,
    pub node_params0: NodeParams,
    pub node_params1: NodeParams,
    pub node_params2: NodeParams,
    pub v_nodes0: Vec<VNodeToken>,
    pub v_nodes1: Vec<VNodeToken>,
    pub v_nodes2: Vec<VNodeToken>,

    pub cdn_node_key0: CdnNodeKey,
    pub cdn_node_key1: CdnNodeKey,
    pub cdn_node_key2: CdnNodeKey,
    pub cdn_node_params0: CdnNodeParams,
    pub cdn_node_params1: CdnNodeParams,
    pub cdn_node_params2: CdnNodeParams,

    pub manager_id: AccountId,
    pub cluster_id: ClusterId,
    pub cluster_params: ClusterParams,
    pub cluster_v_nodes: Vec<VNodeToken>,
    pub nodes_keys: Vec<NodeKey>,
    pub cdn_nodes_keys: Vec<CdnNodeKey>,
    pub rent_per_month: Balance,
    pub capacity: u32,
    pub reserved_resource: u32,
}

pub fn setup_cluster() -> TestCluster {

    let mut contract: DdcBucket = setup_contract();

    let provider_id0 = AccountId::from([0xae, 0x7d, 0xe8, 0x17, 0xa4, 0xa5, 0x12, 0x57, 0xd2, 0x49, 0x64, 0x28, 0x3b, 0x25, 0x69, 0x09, 0xdf, 0x0c, 0x99, 0x97, 0xc0, 0x3e, 0x2b, 0x88, 0x02, 0x02, 0xee, 0x10, 0xf4, 0x4d, 0x72, 0x48]);
    let provider_id1 = AccountId::from([0xc4, 0xba, 0xfd, 0x6a, 0xa1, 0x5a, 0x14, 0xd6, 0xee, 0xf2, 0xea, 0x92, 0xb7, 0xc6, 0x84, 0x51, 0x68, 0x39, 0xbe, 0x96, 0xd6, 0xbf, 0xca, 0xa3, 0x68, 0xd2, 0x4f, 0xff, 0x09, 0x85, 0xa7, 0x1e]);
    let provider_id2 = AccountId::from([0xfa, 0x01, 0x28, 0xf8, 0xe1, 0x32, 0xc6, 0x81, 0x21, 0x06, 0xa5, 0xce, 0xae, 0x6d, 0xcf, 0xf3, 0xd2, 0xc0, 0x1b, 0xb0, 0x13, 0xf2, 0xd7, 0x75, 0x6f, 0x20, 0xf9, 0x50, 0x00, 0xd6, 0xc7, 0x2b]);
    let manager_id = AccountId::from([0xd2, 0xc5, 0xea, 0xa2, 0x0c, 0xd0, 0x4e, 0xfb, 0x3f, 0x10, 0xb8, 0xad, 0xa9, 0xa4, 0x4f, 0xe0, 0x85, 0x41, 0x1f, 0x59, 0xf2, 0x34, 0x1a, 0x92, 0xa3, 0x48, 0x4f, 0x04, 0x51, 0x87, 0x68, 0x54]);

    set_balance(provider_id0, 1000 * TOKEN);
    set_balance(provider_id1, 1000 * TOKEN);
    set_balance(provider_id2, 1000 * TOKEN);
    set_balance(manager_id, 1000 * TOKEN);

    let rent_per_month: Balance = 10 * TOKEN;
    let reserved_resource = 10;
    let capacity = 100;


    // Create the 1st storage node
    let node_key0 = AccountId::from([0x0a; 32]);
    let node_params0 = NodeParams::from("{\"url\":\"https://ddc.cere.network/storage/0\"}");
    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    contract.node_create(
        node_key0,
        node_params0.clone(),
        capacity,
        rent_per_month
    ).unwrap();


    // Create the 2nd storage node
    let node_key1 = AccountId::from([0x0b; 32]);
    let node_params1 = NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/1\"}");
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    contract.node_create(
        node_key1,
        node_params1.clone(),
        capacity,
        rent_per_month
    ).unwrap();


    // Create the 3rd storage node
    let node_key2 = AccountId::from([0x0c; 32]);
    let node_params2 = NodeParams::from("{\"url\":\"https://ddc-2.cere.network/storage/2\"}");
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    let node_key2 = contract.node_create(
        node_key2,
        node_params2.clone(),
        capacity,
        rent_per_month,
    ).unwrap();


    // Create the 1st cdn node
    let cdn_node_key0 = AccountId::from([0x0d; 32]);
    let cdn_node_params0 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/0\"}");
    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key0, 
        cdn_node_params0.clone()
    ).unwrap();


    // Create the 2nd cdn node
    let cdn_node_key1 = AccountId::from([0x0e; 32]);
    let cdn_node_params1 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/1\"}");
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key1, 
        cdn_node_params1.clone()
    ).unwrap();


    // Create the 3rd cdn node
    let cdn_node_key2 = AccountId::from([0x0f; 32]);
    let cdn_node_params2 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/2\"}");
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key2, 
        cdn_node_params2.clone()
    ).unwrap();


    // Create a Cluster
    let cluster_params = ClusterParams::from("{}");
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    let cluster_id = contract.cluster_create(
        cluster_params.clone(),
    ).unwrap();


    // Grant trusted manager_id permission from node providers to the cluster manager_id
    for provider_id in [provider_id0, provider_id1, provider_id2] {
        set_caller_value(provider_id, CONTRACT_FEE_LIMIT);
        contract.grant_trusted_manager_permission(manager_id).unwrap();
        let expected_perm = Permission::ClusterManagerTrustedBy(provider_id);
        assert!(contract.has_permission(manager_id, expected_perm));
    }


    // Add the 1st storage node to the Cluster
    let v_nodes0 = vec![1, 2, 3];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key0, 
        v_nodes0.clone()
    ).unwrap();


    // Add the 2nd storage node to the Cluster
    let v_nodes1 = vec![4, 5, 6];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key1, 
        v_nodes1.clone()
    ).unwrap();


    // Add the 3rd storage node to the Cluster
    let v_nodes2 = vec![7, 8, 9];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key2, 
        v_nodes2.clone()
    ).unwrap();


    // Add the 1st cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key0, 
    ).unwrap();


    // Add the 2nd cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key1, 
    ).unwrap();


    // Add the 3rd cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key2, 
    ).unwrap();


    set_caller(manager_id);
    contract.cluster_reserve_resource(cluster_id, reserved_resource);


    let nodes_keys = vec![
        node_key0,
        node_key1,
        node_key2
    ];

    let cdn_nodes_keys = vec![
        cdn_node_key0,
        cdn_node_key1,
        cdn_node_key2
    ];

    let mut cluster_v_nodes = Vec::<VNodeToken>::new();
    cluster_v_nodes.extend(v_nodes0.clone());
    cluster_v_nodes.extend(v_nodes1.clone());
    cluster_v_nodes.extend(v_nodes2.clone());

    TestCluster {
        contract,

        provider_id0,
        provider_id1,
        provider_id2,

        node_key0,
        node_key1,
        node_key2,
        node_params0,
        node_params1,
        node_params2,
        v_nodes0,
        v_nodes1,
        v_nodes2,

        cdn_node_key0,
        cdn_node_key1,
        cdn_node_key2,
        cdn_node_params0,
        cdn_node_params1,
        cdn_node_params2,

        manager_id,
        cluster_id,
        cluster_params,
        cluster_v_nodes,
        rent_per_month,
        nodes_keys,
        cdn_nodes_keys,
        capacity,
        reserved_resource,
    }
}

pub struct TestBucket {
    pub bucket_id: BucketId,
    pub owner_id: AccountId,
    pub resource: u32,
}

pub fn setup_bucket(ctx: &mut TestCluster) -> TestBucket {
    let owner_id = AccountId::from([0xd4, 0x8f, 0x63, 0x67, 0xe2, 0x15, 0x51, 0xdf, 0x11, 0x1c, 0x92, 0x69, 0x0d, 0x04, 0x3f, 0x75, 0xcb, 0x39, 0xf8, 0x27, 0xbb, 0xc7, 0x46, 0x4b, 0x8d, 0x5d, 0x70, 0xd1, 0x02, 0xaa, 0x71, 0x0a]);
    set_balance(owner_id, 1000 * TOKEN);
    set_caller_value(owner_id, CONTRACT_FEE_LIMIT);

    let bucket_id = ctx.contract.bucket_create(
        "{}".to_string(), 
        ctx.cluster_id, 
        None
    );

    // Reserve some resources for the bucket from the cluster.
    set_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let resource = 1;
    ctx.contract.bucket_alloc_into_cluster(bucket_id, resource);

    // Deposit some value to pay for buckets.
    set_caller_value(owner_id, 10 * TOKEN);
    ctx.contract.account_deposit();

    TestBucket {
        bucket_id,
        owner_id,
        resource,
    }
}