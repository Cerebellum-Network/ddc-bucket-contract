use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, ClusterCreated, ClusterVNodeReplaced, DdcBucket, Result};
use crate::ddc_bucket::cluster::entity::{Cluster, PartitionIndex};
use crate::ddc_bucket::Error::{PartitionDoesNotExist, UnauthorizedClusterManager};
use crate::ddc_bucket::vnode::entity::VNodeId;

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(&mut self, manager: AccountId, cluster_params: ClusterParams) -> Result<ClusterId> {
        let (cluster_id, record_size) = self.clusters.create(manager, cluster_params.clone());
        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(ClusterCreated { cluster_id, manager, cluster_params });
        Ok(cluster_id)
    }

    pub fn message_cluster_replace_vnode(&mut self, cluster_id: ClusterId, partition_index: PartitionIndex, new_vnode_id: VNodeId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;
        let vnode_id = cluster.vnode_ids.get_mut(partition_index as usize).ok_or(PartitionDoesNotExist)?;
        *vnode_id = new_vnode_id;
        Self::env().emit_event(ClusterVNodeReplaced { cluster_id, vnode_id: new_vnode_id, partition_index });
        Ok(())
    }

    fn only_cluster_manager(cluster: &Cluster) -> Result<()> {
        let caller = Self::env().caller();
        if caller == cluster.manager {
            Ok(())
        } else {
            Err(UnauthorizedClusterManager)
        }
    }
}
