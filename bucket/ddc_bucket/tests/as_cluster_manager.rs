use crate::ddc_bucket::cluster::entity::{PartitionId, PartitionIndex};
use crate::ddc_bucket::DdcBucket;
use crate::ddc_bucket::vnode::entity::VNodeId;

pub fn replace_node(contract: &mut DdcBucket, old_vnode_id: VNodeId, new_vnode_id: VNodeId) {
    let partition_ids = find_partitions_of_node(contract, old_vnode_id);

    for (cluster_id, partition_i) in partition_ids.iter() {
        contract.cluster_replace_vnode(*cluster_id, *partition_i, new_vnode_id).unwrap();
    }
}


pub fn find_partitions_of_node(contract: &DdcBucket, vnode_id: VNodeId) -> Vec<PartitionId> {
    let mut partition_ids = Vec::new();

    // Discover the available clusters.
    let (clusters, _count) = contract.cluster_list(0, 20);
    if _count > 20 { unimplemented!("full cluster iteration") }

    for cluster in clusters.iter() {
        for (index, &some_vnode_id) in cluster.vnode_ids.iter().enumerate() {
            if some_vnode_id == vnode_id {
                let partition_id = (cluster.cluster_id, index as PartitionIndex);
                partition_ids.push(partition_id);
            }
        }
    }

    partition_ids
}
