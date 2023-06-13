//! The public interface to manage Clusters.
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cluster::entity::{Cluster, ClusterStatus};
use crate::ddc_bucket::node::entity::{Node, NodeKey, Resource};
use crate::ddc_bucket::cdn_node::entity::{CdnNodeKey};
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
        nodes_keys: Vec<NodeKey>,
        v_nodes: Vec<Vec<u64>>,
        cdn_nodes_keys: Vec<CdnNodeKey>,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let manager = Self::env().caller();

        let mut nodes = Vec::<(NodeKey, Node)>::new();
        for node_key in &nodes_keys {
            let node = self.nodes.get(*node_key)?;
            // Verify that the node provider trusts the cluster manager.
            Self::only_trusted_manager(&self.perms, manager, node.provider_id.clone())?;
            nodes.push((*node_key, node));
        }

        let cluster_id = self.clusters.create(manager, &v_nodes, &nodes_keys)?;
        let rent = self
            .topology_store
            .create_topology(cluster_id, v_nodes, nodes.into_iter().map(|(key, node)| (key, node)).collect())?;

        self.clusters.get_mut(cluster_id).unwrap().change_rent(rent);

        let params_id = self.cluster_params.create(cluster_params.clone())?;
        assert_eq!(cluster_id, params_id);

        Self::env().emit_event(ClusterCreated {
            cluster_id,
            manager,
            cluster_params,
        });
        Ok(cluster_id)
    }

    pub fn message_cluster_add_node(
        &mut self,
        cluster_id: ClusterId,
        node_keys: Vec<NodeKey>,
        v_nodes: Vec<Vec<u64>>,
    ) -> Result<()> {
        let manager = Self::env().caller();
        let mut nodes = Vec::<(NodeKey, Node)>::new();

        for node_key in &node_keys {
            let node = self.nodes.get(*node_key)?;
            // Verify that the node provider trusts the cluster manager.
            Self::only_trusted_manager(&self.perms, manager, node.provider_id.clone())?;
            nodes.push((*node_key, node));
        }

        // add node and redistribute v_nodes
        let cluster = self.clusters.get(cluster_id)?;

        let mut old_v_nodes = Vec::<u64>::new();
        for v_nodes_per_phys_node in &cluster.v_nodes {
            for v_node in v_nodes_per_phys_node {
                old_v_nodes.push(*v_node);
            }
        }

        // TODO: change v_nodes inside cluster entity
        let total_rent = self
            .topology_store
            .add_node(cluster_id, &old_v_nodes, &v_nodes, nodes.into_iter().map(|(key, node)| (key, node)).collect())?;

        let cluster = self.clusters.get_mut(cluster_id)?;
        cluster.total_rent = total_rent as Balance;
        cluster.v_nodes = v_nodes;
        cluster.node_keys = node_keys;

        Ok(())
    }

    pub fn message_cluster_reserve_resource(
        &mut self,
        cluster_id: ClusterId,
        resource: Resource,
    ) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        Self::only_cluster_manager(cluster)?;
        cluster.put_resource(resource);

        for v_nodes_wrapper in &cluster.v_nodes {
            for &v_node in v_nodes_wrapper {
                let node_key = self.topology_store.get(cluster_id, v_node)?;
                let mut node = self.nodes.get(node_key)?;
                node.take_resource(resource)?;
                self.nodes.update(node_key, &node)?;
            }
        }

        Self::env().emit_event(ClusterReserveResource {
            cluster_id,
            resource,
        });
        Ok(())
    }

    // v_nodes length should be equal to v_nodes which were assigned to a physical node before
    pub fn message_cluster_replace_node(
        &mut self,
        cluster_id: ClusterId,
        v_nodes: Vec<u64>,
        new_node_key: NodeKey,
    ) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;
        let manager = Self::only_cluster_manager(cluster)?;

        // Give back resources to the old node for all its v_nodes
        for v_node in v_nodes.clone() {
            let old_node_key = self.topology_store.get(cluster_id, v_node)?;

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

            self.topology_store.save(cluster_id, v_node, new_node_key);
        }

        self.topology_store
            .replace_node(cluster_id, v_nodes.clone(), new_node_key)?;

        cluster.replace_v_node(v_nodes, new_node_key);

        Self::env().emit_event(ClusterNodeReplaced {
            cluster_id,
            node_key: new_node_key,
        });
        Ok(())
    }

    pub fn message_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) -> Result<()> {
        let cluster = self.clusters.get_mut(cluster_id)?;

        // Charge the network fee from the cluster.
        Self::capture_network_fee(&self.network_fee, &mut cluster.revenues)?;

        // Charge the cluster management fee.
        Self::capture_fee(
            self.network_fee.cluster_management_fee_bp(),
            cluster.manager_id,
            &mut cluster.revenues,
        )?;

        // Charge the provider payments from the cluster.
        let num_shares = cluster.v_nodes.len() as Balance;
        let per_share = cluster.revenues.peek() / num_shares;
        cluster.revenues.pay(Payable(per_share * num_shares))?;

        for node_key in &cluster.node_keys {
            let node = self.nodes.get(*node_key)?;
            Self::send_cash(node.provider_id, Cash(per_share))?;

            Self::env().emit_event(ClusterDistributeRevenues {
                cluster_id,
                provider_id: node.provider_id,
            });
        }
        // TODO: set a maximum node count, or support paging.
        // TODO: aggregate the payments per node_id or per provider_id.

        Ok(())
    }

    pub fn message_cluster_change_params(
        &mut self,
        cluster_id: ClusterId,
        params: ClusterParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;

        Self::impl_change_params(&mut self.cluster_params, cluster_id, params)
    }

    pub fn message_cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterStatus> {
        let cluster = self.clusters.get(cluster_id)?.clone();
        let params = self.cluster_params.get(cluster_id)?.clone();
        Ok(ClusterStatus {
            cluster_id,
            cluster,
            params,
        })
    }

    pub fn message_cluster_list(
        &self,
        offset: u32,
        limit: u32,
        filter_manager_id: Option<AccountId>,
    ) -> (Vec<ClusterStatus>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.clusters.0.get(cluster_id as usize) {
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
        (clusters, self.clusters.0.len().try_into().unwrap())
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
