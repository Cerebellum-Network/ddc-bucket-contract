//! The public interface to manage Clusters.
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cluster::entity::{Cluster, ClusterInfo};
use crate::ddc_bucket::node::entity::{Node, NodeKey, Resource};
use crate::ddc_bucket::cdn_node::entity::{CdnNode, CdnNodeKey};
use crate::ddc_bucket::topology::store::{VNodeToken};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::perm::store::PermStore;
use crate::ddc_bucket::ClusterNodeReplaced;
use crate::ddc_bucket::Error::{ClusterManagerIsNotTrusted, UnauthorizedClusterManager};
use crate::ddc_bucket::{
    AccountId, Balance, ClusterCreated, ClusterDistributeRevenues, ClusterReserveResource,
    DdcBucket, Result,
};

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {

    pub fn message_cluster_create(
        &mut self,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let caller = Self::env().caller();

        let cluster_id = self.clusters.create(
            caller,
            cluster_params.clone()
        )?;
        
        self.topology_store.create_topology(cluster_id)?;

        Self::env().emit_event(ClusterCreated {
            cluster_id,
            manager: caller,
            cluster_params,
        });

        Ok(cluster_id)
    }

    pub fn message_cluster_add_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey,
        v_nodes: Vec<VNodeToken>,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut node: Node = self.nodes.get(node_key)?;
        node.only_without_cluster();
        Self::only_trusted_manager(&self.perms, caller, node.provider_id)?;

        node.cluster_id = Some(cluster_id);
        self.nodes.update(node_key, &node);

        let mut cluster: Cluster = self.clusters.get(cluster_id)?;
        cluster.nodes_keys.push(node_key);
        for v_node in v_nodes.iter() {
            cluster.total_rent += node.rent_per_month;
        }
        self.clusters.update(cluster_id, &cluster)?;

        self.topology_store.add_node(cluster_id, node_key, v_nodes)?;

        Ok(())
    }

    pub fn message_cluster_reserve_resource(
        &mut self,
        cluster_id: ClusterId,
        resource: Resource,
    ) -> Result<()> {
        let mut cluster = self.clusters.get(cluster_id)?;
        Self::only_cluster_manager(&cluster)?;
        cluster.put_resource(resource);
        self.clusters.update(cluster_id, &cluster)?;

        let cluster_vnodes = self.topology_store.get_vnodes_by_cluster(cluster_id)?;
        for v_node in cluster_vnodes {
            let node_key = self.topology_store.get_node_by_vnode(cluster_id, v_node)?;
            let mut node = self.nodes.get(node_key)?;
            node.take_resource(resource)?;
            self.nodes.update(node_key, &node)?;
        }

        Self::env().emit_event(ClusterReserveResource {
            cluster_id,
            resource,
        });
        Ok(())
    }

    pub fn message_cluster_replace_node(
        &mut self,
        cluster_id: ClusterId,
        v_nodes: Vec<u64>,
        new_node_key: NodeKey,
    ) -> Result<()> {
        let cluster = self.clusters.get(cluster_id)?;
        let manager = Self::only_cluster_manager(&cluster)?;

        // Give back resources to the old node for all its v_nodes
        for v_node in v_nodes.clone() {
            let old_node_key = self.topology_store.get_node_by_vnode(cluster_id, v_node)?;

            // Give back resources to the old node
            let mut old_node = self.nodes.get(old_node_key)?;
            old_node.put_resource(cluster.resource_per_vnode);
            self.nodes.update(old_node_key, &old_node)?;

            let mut new_node = self.nodes.get(new_node_key)?;
            // Verify that the provider of the new node trusts the cluster manager.
            Self::only_trusted_manager(&self.perms, manager, new_node.provider_id)?;
            // Reserve resources on the new node.
            new_node.take_resource(cluster.resource_per_vnode)?;
            self.nodes.update(new_node_key, &new_node)?;
        }

        self.topology_store
            .replace_node(cluster_id, new_node_key, v_nodes)?;

        Self::env().emit_event(ClusterNodeReplaced {
            cluster_id,
            node_key: new_node_key,
        });

        Ok(())
    }

    pub fn message_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) -> Result<()> {
        let mut cluster = self.clusters.get(cluster_id)?;

        // Charge the network fee from the cluster.
        Self::capture_network_fee(&self.network_fee, &mut cluster.revenues)?;

        // Charge the cluster management fee.
        Self::capture_fee(
            self.network_fee.cluster_management_fee_bp(),
            cluster.manager_id,
            &mut cluster.revenues,
        )?;

        // Charge the provider payments from the cluster.
        let cluster_vnodes = self.topology_store.get_vnodes_by_cluster(cluster_id)?;
        let num_shares = cluster_vnodes.len() as Balance;
        let per_share = cluster.revenues.peek() / num_shares;
        cluster.revenues.pay(Payable(per_share * num_shares))?;

        for node_key in &cluster.nodes_keys {
            let node = self.nodes.get(*node_key)?;
            Self::send_cash(node.provider_id, Cash(per_share))?;

            Self::env().emit_event(ClusterDistributeRevenues {
                cluster_id,
                provider_id: node.provider_id,
            });
        }

        self.clusters.update(cluster_id, &cluster)?;

        // TODO: set a maximum node count, or support paging.
        // TODO: aggregate the payments per node_id or per provider_id.

        Ok(())
    }

    pub fn message_cluster_set_params(
        &mut self,
        cluster_id: ClusterId,
        cluster_params: ClusterParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let mut cluster: Cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;
        cluster.cluster_params = cluster_params;
        self.clusters.update(cluster_id, &cluster)?;
        Ok(())
    }

    pub fn message_cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterInfo> {
        let cluster = self.clusters.get(cluster_id)?.clone();
        Ok(ClusterInfo {
            cluster_id,
            cluster,
        })
    }

    pub fn message_cluster_list(
        &self,
        offset: u32,
        limit: u32,
        filter_manager_id: Option<AccountId>,
    ) -> (Vec<ClusterInfo>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.clusters.clusters.get(cluster_id) {
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
            let status = ClusterInfo {
                cluster_id,
                cluster: cluster.clone(),
            };
            clusters.push(status);
        }
        (clusters, self.clusters.next_cluster_id - 1)
    }

    fn only_cluster_manager(cluster: &Cluster) -> Result<AccountId> {
        let caller = Self::env().caller();
        if caller == cluster.manager_id {
            Ok(caller)
        } else {
            Err(UnauthorizedClusterManager)
        }
    }

    fn only_trusted_manager(
        perm_store: &PermStore,
        manager: AccountId,
        trusted_by: AccountId,
    ) -> Result<()> {
        let perm = Permission::ManagerTrustedBy(trusted_by);
        let trusts = perm_store.has_permission(manager, perm);
        if trusts {
            Ok(())
        } else {
            Err(ClusterManagerIsNotTrusted)
        }
    }
}
