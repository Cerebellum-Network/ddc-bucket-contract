//! The public interface to manage Clusters.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, ClusterCreated, ClusterNodeReplaced, DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cluster::entity::{Cluster, PartitionIndex};
use crate::ddc_bucket::Error::{PartitionDoesNotExist, UnauthorizedClusterManager};
use crate::ddc_bucket::node::entity::{NodeId, Resource};

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(
        &mut self,
        manager: AccountId,
        partition_count: u32,
        node_ids: Vec<NodeId>,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let mut nodes = Vec::new();
        for node_id in &node_ids {
            let node = self.nodes.get(*node_id)?;
            nodes.push(node);
        }
        let (cluster_id, record_size) = self.clusters.create(manager, partition_count, &nodes, cluster_params.clone());
        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(ClusterCreated { cluster_id, manager, cluster_params });
        Ok(cluster_id)
    }

    pub fn message_cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        cluster.put_resource(amount);

        for node_id in &cluster.vnodes {
            let node = self.nodes.get_mut(*node_id)?;
            node.take_resource(amount)?;
        }

        Ok(())
    }

    pub fn message_cluster_replace_node(&mut self, cluster_id: ClusterId, partition_index: PartitionIndex, new_node_id: NodeId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;

        // Give back resources to the old node.
        let old_node_id = cluster.vnodes.get_mut(partition_index as usize).ok_or(PartitionDoesNotExist)?;

        self.nodes.get_mut(*old_node_id)?
            .put_resource(cluster.resource_per_vnode);

        // Reserve resources on the new node.
        self.nodes.get_mut(new_node_id)?
            .take_resource(cluster.resource_per_vnode)?;
        *old_node_id = new_node_id;

        Self::env().emit_event(ClusterNodeReplaced { cluster_id, node_id: new_node_id, partition_index });
        Ok(())
    }

    pub fn message_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        let num_shares = cluster.vnodes.len() as Balance;
        let per_share = cluster.revenues.peek() / num_shares;
        cluster.revenues.pay(Payable(per_share * num_shares))?;

        for node_id in &cluster.vnodes {
            let node = self.nodes.get(*node_id)?;
            Self::send_cash(node.provider_id, Cash(per_share))?;
        }

        // TODO: set a maximum node count, or support paging.
        // TODO: aggregate the payments per node_id or per provider_id.

        Ok(())
    }

    fn only_cluster_manager(cluster: &Cluster) -> Result<()> {
        let caller = Self::env().caller();
        if caller == cluster.manager_id {
            Ok(())
        } else {
            Err(UnauthorizedClusterManager)
        }
    }
}
