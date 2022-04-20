//! The public interface to manage Clusters.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, ClusterCreated, ClusterNodeReplaced, DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cluster::entity::{Cluster, ClusterStatus, VNodeIndex};
use crate::ddc_bucket::Error::{ClusterManagerIsNotTrusted, VNodeDoesNotExist, UnauthorizedClusterManager};
use crate::ddc_bucket::node::entity::{Node, NodeId, Resource};
use crate::ddc_bucket::perm::entity::Permission;

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(
        &mut self,
        manager: AccountId,
        vnode_count: u32,
        node_ids: Vec<NodeId>,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let mut nodes = Vec::<(NodeId, &Node)>::new();
        for node_id in node_ids {
            let node = self.nodes.get(node_id)?;
            nodes.push((node_id, node));

            // Verify that the node provider trusts the cluster manager.
            let perm = Permission::ManagerTrustedBy(node.provider_id);
            let trusts = self.perms.has_permission(manager, perm);
            if !trusts { return Err(ClusterManagerIsNotTrusted); }
        }

        let (cluster_id, record_size0) = self.clusters.create(manager, vnode_count, &nodes)?;
        let (params_id, recorde_size1) = self.cluster_params.create(cluster_params.clone())?;
        assert_eq!(cluster_id, params_id);

        Self::capture_fee_and_refund(record_size0 + recorde_size1)?;
        Self::env().emit_event(ClusterCreated { cluster_id, manager, cluster_params });
        Ok(cluster_id)
    }

    pub fn message_cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;
        cluster.put_resource(amount);

        for node_id in &cluster.vnodes {
            let node = self.nodes.get_mut(*node_id)?;
            node.take_resource(amount)?;
        }

        Ok(())
    }

    pub fn message_cluster_replace_node(&mut self, cluster_id: ClusterId, vnode_index: VNodeIndex, new_node_id: NodeId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;

        // Give back resources to the old node.
        let old_node_id = cluster.vnodes.get_mut(vnode_index as usize).ok_or(VNodeDoesNotExist)?;

        self.nodes.get_mut(*old_node_id)?
            .put_resource(cluster.resource_per_vnode);

        // Reserve resources on the new node.
        self.nodes.get_mut(new_node_id)?
            .take_resource(cluster.resource_per_vnode)?;
        *old_node_id = new_node_id;

        Self::env().emit_event(ClusterNodeReplaced { cluster_id, node_id: new_node_id, vnode_index });
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

    pub fn message_cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterStatus> {
        let cluster = self.clusters.get(cluster_id)?.clone();
        let params = self.cluster_params.get(cluster_id)?.clone();
        Ok(ClusterStatus { cluster_id, cluster, params })
    }

    pub fn message_cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<ClusterStatus>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.clusters.0.get(cluster_id) {
                None => break, // No more items, stop.
                Some(cluster) => cluster,
            };
            // Apply the filter if given.
            if let Some(manager_id) = filter_manager_id {
                if manager_id != cluster.manager_id {
                    continue; // Skip non-matches.
                }
            }
            // Include the complete status of matched items.
            let status = ClusterStatus {
                cluster_id,
                cluster: cluster.clone(),
                params: self.cluster_params.get(cluster_id).unwrap().clone(),
            };
            clusters.push(status);
        }
        (clusters, self.clusters.0.len())
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
