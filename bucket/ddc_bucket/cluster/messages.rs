use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, ClusterCreated, ClusterNodeReplaced, DdcBucket, Result};
use crate::ddc_bucket::cluster::entity::{Cluster, PartitionIndex};
use crate::ddc_bucket::Error::{PartitionDoesNotExist, UnauthorizedClusterManager};
use crate::ddc_bucket::node::entity::NodeId;

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(
        &mut self,
        manager: AccountId,
        partition_count: u32,
        node_ids: Vec<NodeId>,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let (cluster_id, record_size) = self.clusters.create(manager, partition_count, node_ids, cluster_params.clone());
        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(ClusterCreated { cluster_id, manager, cluster_params });
        Ok(cluster_id)
    }

    pub fn message_cluster_replace_node(&mut self, cluster_id: ClusterId, partition_index: PartitionIndex, new_node_id: NodeId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;
        let node_id = cluster.vnodes.get_mut(partition_index as usize).ok_or(PartitionDoesNotExist)?;
        *node_id = new_node_id;
        Self::env().emit_event(ClusterNodeReplaced { cluster_id, node_id: new_node_id, partition_index });
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
