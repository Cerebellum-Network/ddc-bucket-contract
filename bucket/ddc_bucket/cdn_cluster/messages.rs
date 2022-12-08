//! The public interface to manage Clusters.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, BucketId, CdnClusterCreated, ClusterDistributeRevenues, ClusterId, DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cdn_cluster::entity::{CdnCluster, CdnClusterStatus};
use crate::ddc_bucket::Error::{ClusterManagerIsNotTrusted, UnauthorizedClusterManager, InsufficientBalance};
use crate::ddc_bucket::cdn_node::entity::{CdnNode, NodeId, Resource};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::perm::store::PermStore;

const KB_PER_GB: Balance = 1_000_000;

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

    // Set the price usd per gb
    pub fn message_cdn_set_rate(&mut self, cluster_id: ClusterId, usd_per_gb: u128) -> Result<()> {
        let cluster = self.cdn_clusters.get_mut(cluster_id)?;
        Self::only_cdn_cluster_manager(cluster)?;

        cluster.set_rate(usd_per_gb)?;

        Ok(())
    }

    // Get the price usd per gb
    pub fn message_cdn_get_rate(&self, cluster_id: ClusterId) -> Result<Balance> {
        let cluster = self.cdn_clusters.get(cluster_id)?;
        let rate = cluster.get_rate();
        Ok(rate)
    }

    // First payment is for aggregate consumption for account, second is the aggregate payment for the node (u32 for ids)
    pub fn message_cdn_cluster_put_revenue(&mut self, cluster_id: ClusterId, aggregates_accounts: Vec<(AccountId, u128)>, aggregates_nodes: Vec<(u32, u128)>, aggregates_buckets: Vec<(BucketId, Resource)>) -> Result<()> {
        let cluster = self.cdn_clusters.get_mut(cluster_id)?;
        // Self::only_cdn_cluster_manager(cluster)?;

        let mut cluster_payment = 0;
        let mut _undistributed_payment_accounts = 0;

        let aggregate_payments_accounts;
        {
            let conv = &self.accounts.1;
            aggregate_payments_accounts = aggregates_accounts.iter().map(|(client_id, resources_used)| {
                let account_id = *client_id;
                let cere_payment: Balance = conv.to_cere(*resources_used as Balance * cluster.usd_per_gb / KB_PER_GB );
                (account_id, cere_payment)
            }).collect::<Vec<(AccountId, Balance)>>();
        }

        for &(client_id, payment) in aggregate_payments_accounts.iter() {
            let account = self.accounts.0.get_mut(&client_id)
            .ok_or(InsufficientBalance)?;
            account.withdraw_bonded(Payable(payment))?;
            _undistributed_payment_accounts += payment;
        }

        let conv = &self.accounts.1;

        for &(node_id, resources_used) in aggregates_nodes.iter() {
            let node = self.cdn_nodes.get_mut(node_id)?;

            let payment = conv.to_cere (resources_used as Balance * cluster.usd_per_gb / KB_PER_GB );

            node.put_payment(payment);
            cluster_payment += payment;
        }
        // Add check that two payments should equal?

        // Go through buckets and deduct used resources
        for &(bucket_id, resources_used) in aggregates_buckets.iter() {
            let bucket = self.buckets.get_mut(bucket_id)?;

            if bucket.resource_consumption_cap <= resources_used {
                bucket.resource_consumption_cap -= resources_used;
            }
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
