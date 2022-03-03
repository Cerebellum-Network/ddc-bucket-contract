use ink_lang as ink;

use crate::ddc_bucket::*;

use super::{as_gateway::*, as_storage::*, as_user::*, test_utils::*};

#[ink::test]
fn partitioning_works() {
    let _ddc_bucket = setup_cluster()?;
}

fn setup_cluster() -> Result<DdcBucket> {
    let accounts = get_accounts();

    let mut ddc_bucket = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Create a storage Cluster and a gateway Cluster.
    push_caller(accounts.alice);
    let storage_cluster_id = ddc_bucket.cluster_create("engine storage".to_string())?;
    let gateway_cluster_id = ddc_bucket.cluster_create("engine gateway".to_string())?;
    pop_caller();

    // Provide one gateway VNode.
    let mut gateway_node = TestGateway::new(accounts.alice, "alice");
    gateway_node.vnode.join_cluster(&mut ddc_bucket, gateway_cluster_id)?;

    // Provide two storage VNode’s.
    let mut storage_node0 = TestStorage::new(accounts.bob, "bob");
    storage_node0.vnode.join_cluster(&mut ddc_bucket, storage_cluster_id)?;

    let mut storage_node1 = TestStorage::new(accounts.charlie, "charlie");
    storage_node1.vnode.join_cluster(&mut ddc_bucket, storage_cluster_id)?;

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut ddc_bucket, accounts.django)?;

    // Simulate a request.
    let request = user.make_request(&ddc_bucket)?;
    gateway_node.handle_request(request)?;

    Ok(ddc_bucket)
}
