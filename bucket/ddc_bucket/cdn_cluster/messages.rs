//! The public interface to manage Clusters.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, CdnClusterCreated, ClusterDistributeRevenues, DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cdn_cluster::entity::{CdnCluster, CdnClusterStatus};
use crate::ddc_bucket::Error::{ClusterManagerIsNotTrusted, UnauthorizedClusterManager, InsufficientBalance};
use crate::ddc_bucket::cdn_node::entity::{CdnNode, NodeId};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::perm::store::PermStore;

use super::entity::{ClusterId};

impl DdcBucket {
    pub fn message_cdn_cluster_create(
        &mut self,
        cdn_nodes_ids: Vec<NodeId>,
    ) -> Result<ClusterId> {
        let manager = Self::env().caller();

        let cdn_nodes = cdn_nodes_ids.clone();

        let mut nodes = Vec::<(NodeId, &CdnNode)>::new();
        for node_id in cdn_nodes_ids {
            let node = self.cdn_nodes.get(node_id)?;
            nodes.push((node_id, node));

            // Verify that the node provider trusts the cluster manager.
            Self::only_cdn_trusted_manager(&self.perms, manager, node.provider_id)?;
        }

        let (cluster_id, record_size0) = self.cdn_clusters.create(manager, cdn_nodes)?;

        Self::capture_fee_and_refund(record_size0)?;
        Self::env().emit_event(CdnClusterCreated { cluster_id, manager });
        Ok(cluster_id)
    }

    pub fn message_cdn_cluster_put_revenue(&mut self, cluster_id: ClusterId) -> Result<()> {
        let cluster = self.cdn_clusters.get_mut(cluster_id)?;
        Self::only_cdn_cluster_manager(cluster)?;

        let mut cluster_payment = 0;

        for node_id in &cluster.cdn_nodes {
          let node = self.cdn_nodes.get_mut(*node_id)?;
          let node_provider = node.provider_id.clone();
          let mut undistributed_payment = 0;
          // get logs per provider
          let logs = self.committer_store.client_logs.get(&node_provider).unwrap();
          for log in logs {
            let (_cdn_id, client_id, resources_used, _timestamp) = log;
            // take payment from the clients
            let account = self.accounts.0.get_mut(&client_id)
            .ok_or(InsufficientBalance)?;

            let conv = &self.accounts.1;
            let cere_payment = conv.to_cere (*resources_used as Balance * USD_PER_GB / 1000000 );
            account.withdraw_bonded(Payable(cere_payment))?;
            undistributed_payment += cere_payment;
          }
          // Add revenues to nodes
          node.put_payment(undistributed_payment);
          cluster_payment += undistributed_payment;
        }

        // Add revenues to cluster
        cluster.put_revenues(Cash(cluster_payment));

        Ok(())
    }

    pub fn message_cdn_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) -> Result<()> {
        let cluster = self.cdn_clusters.get_mut(cluster_id)?;

        // Charge the network fee from the cluster.
        Self::capture_network_fee(&self.network_fee, &mut cluster.revenues)?;

        // Charge the cluster management fee.
        Self::capture_fee(
            self.network_fee.cluster_management_fee_bp(),
            cluster.manager_id,
            &mut cluster.revenues)?;

        // First accumulated revenues to distribute.
        let mut distributed_revenue = 0;
  
        for node_id in &cluster.cdn_nodes {
            let node = self.cdn_nodes.get(*node_id)?;
            distributed_revenue += node.undistributed_payment;
        }

        // Charge the provider payments from the cluster.
        cluster.revenues.pay(Payable(distributed_revenue))?;

        // Distribute revenues to nodes
        for node_id in &cluster.cdn_nodes {
            let node = self.cdn_nodes.get_mut(*node_id)?;

            Self::send_cash(node.provider_id, Cash(node.undistributed_payment))?;
            node.take_payment(node.undistributed_payment)?;
            Self::env().emit_event(ClusterDistributeRevenues { cluster_id, provider_id: node.provider_id });
        }

        Ok(())
    }

    pub fn message_cdn_cluster_get(&self, cluster_id: ClusterId) -> Result<CdnClusterStatus> {
        let cluster = self.cdn_clusters.get(cluster_id)?.clone();
        Ok(CdnClusterStatus { cluster_id, cluster })
    }

    pub fn message_cdn_cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<CdnClusterStatus>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.cdn_clusters.0.get(cluster_id) {
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
            let status = CdnClusterStatus {
                cluster_id,
                cluster: cluster.clone(),
            };
            clusters.push(status);
        }
        (clusters, self.clusters.0.len())
    }

    fn only_cdn_cluster_manager(cluster: &CdnCluster) -> Result<AccountId> {
        let caller = Self::env().caller();
        if caller == cluster.manager_id {
            Ok(caller)
        } else {
            Err(UnauthorizedClusterManager)
        }
    }

    fn only_cdn_trusted_manager(perm_store: &PermStore, manager: AccountId, trusted_by: AccountId) -> Result<()> {
        let perm = Permission::ManagerTrustedBy(trusted_by);
        let trusts = perm_store.has_permission(manager, perm);
        if trusts { Ok(()) } else { Err(ClusterManagerIsNotTrusted) }
    }
}

pub const USD_PER_GB: u128 = 1;