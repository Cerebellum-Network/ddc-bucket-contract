//! The public interface to manage Clusters.
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::cluster::entity::{ClusterInfo, KB_PER_GB};
use crate::ddc_bucket::bucket::entity::{BucketId};
use crate::ddc_bucket::node::entity::{Node, NodeKey, Resource};
use crate::ddc_bucket::cdn_node::entity::{CdnNode, CdnNodeKey};
use crate::ddc_bucket::topology::store::{VNodeToken};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::ClusterNodeReplaced;
use crate::ddc_bucket::{
    BASIS_POINTS,
    AccountId, Balance, ClusterCreated, ClusterNodeAdded, ClusterNodeRemoved,
    ClusterCdnNodeAdded, ClusterCdnNodeRemoved, ClusterDistributeRevenues, ClusterReserveResource, ClusterDistributeCdnRevenues,
    ClusterRemoved, ClusterParamsSet, ClusterNodeStatusSet, ClusterCdnNodeStatusSet, PermissionGranted, PermissionRevoked, ClusterNodeReset,
    DdcBucket, NodeStatusInCluster, Result, Error::*
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
        
        self.topology.create_topology(cluster_id)?;

        Self::env().emit_event(ClusterCreated {
            cluster_id,
            manager_id: caller,
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
        node.only_without_cluster()?;
        self.only_trusted_cluster_manager(node.provider_id)?;

        let mut cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;

        node.set_cluster(cluster_id, NodeStatusInCluster::ADDING);
        self.nodes.update(node_key, &node)?;

        cluster.add_node(node_key)?;
        for _v_node in &v_nodes {
            node.reserve_resource(cluster.resource_per_v_node)?;
            cluster.total_rent += node.rent_per_month;
        }
        self.clusters.update(cluster_id, &cluster)?;

        self.topology.add_node(cluster_id, node_key, v_nodes)?;

        Self::env().emit_event(ClusterNodeAdded { 
            cluster_id, 
            node_key 
        });

        Ok(())
    }


    pub fn message_cluster_remove_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut node = self.nodes.get(node_key)?;
        let mut cluster = self.clusters.get(cluster_id)?;

        if !cluster.only_manager(caller).is_ok()
            && !node.only_provider(caller).is_ok() {
                return Err(OnlyClusterManagerOrNodeProvider);
        }

        node.only_with_cluster(cluster_id)?;
        
        node.unset_cluster();
        self.nodes.update(node_key, &node)?;

        cluster.remove_node(node_key);
        let v_nodes = self.topology.get_v_nodes_by_node(node_key);
        for _v_node in &v_nodes {
            node.release_resource(cluster.resource_per_v_node);
            cluster.total_rent -= node.rent_per_month;
        }
        self.clusters.update(cluster_id, &cluster)?;

        self.topology.remove_node(cluster_id, node_key)?;

        Self::env().emit_event(ClusterNodeRemoved { 
            cluster_id, 
            node_key 
        });

        Ok(())
    }


    pub fn message_cluster_replace_node(
        &mut self,
        cluster_id: ClusterId,
        v_nodes: Vec<VNodeToken>,
        new_node_key: NodeKey,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let cluster = self.clusters.get(cluster_id)?;
        let mut new_node = self.nodes.get(new_node_key)?;
        new_node.only_with_cluster(cluster_id)?;
        cluster.only_manager(caller)?;

        // Give back resources to the old node for all its v_nodes
        for v_node in &v_nodes {
            let old_node_key = self.topology.get_node_by_v_node(cluster_id, *v_node)?;

            // Give back resources to the old node
            let mut old_node = self.nodes.get(old_node_key)?;
            old_node.release_resource(cluster.resource_per_v_node);
            self.nodes.update(old_node_key, &old_node)?;

            // Reserve resources on the new node.
            new_node.reserve_resource(cluster.resource_per_v_node)?;
            self.nodes.update(new_node_key, &new_node)?;
        }

        self.topology.replace_node(
            cluster_id, 
            new_node_key, 
            v_nodes
        )?;

        Self::env().emit_event(ClusterNodeReplaced {
            cluster_id,
            node_key: new_node_key,
        });

        Ok(())
    }


    pub fn message_cluster_reset_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey,
        new_v_nodes: Vec<VNodeToken>,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut cluster = self.clusters.get(cluster_id)?;

        let mut node = self.nodes.get(node_key)?;
        node.only_with_cluster(cluster_id)?;
        cluster.only_manager(caller)?;

        let old_v_nodes = self.topology.get_v_nodes_by_node(node_key);

        if new_v_nodes.len() > old_v_nodes.len() {
            
            for _i in 0..new_v_nodes.len() - old_v_nodes.len() {
                node.reserve_resource(cluster.resource_per_v_node)?;
                cluster.total_rent += node.rent_per_month;
            }

            self.nodes.update(node_key, &node)?;
            self.clusters.update(cluster_id, &cluster)?;

        } else if new_v_nodes.len() < old_v_nodes.len() {

            for _i in 0..old_v_nodes.len() - new_v_nodes.len() {
                node.release_resource(cluster.resource_per_v_node);
                cluster.total_rent -= node.rent_per_month;
            }

            self.nodes.update(node_key, &node)?;
            self.clusters.update(cluster_id, &cluster)?;
        }

        self.topology.reset_node(
            cluster_id, 
            node_key, 
            new_v_nodes
        )?;

        Self::env().emit_event(ClusterNodeReset {
            cluster_id,
            node_key: node_key,
        });

        Ok(())
    }


    pub fn message_cluster_add_cdn_node(
        &mut self,
        cluster_id: ClusterId,
        cdn_node_key: CdnNodeKey,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut cdn_node: CdnNode = self.cdn_nodes.get(cdn_node_key)?;
        cdn_node.only_without_cluster()?;
        self.only_trusted_cluster_manager(cdn_node.provider_id)?;

        let mut cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;

        cdn_node.set_cluster(cluster_id, NodeStatusInCluster::ADDING);
        self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

        cluster.add_cdn_node(cdn_node_key)?;
        self.clusters.update(cluster_id, &cluster)?;

        Self::env().emit_event(ClusterCdnNodeAdded { 
            cluster_id, 
            cdn_node_key 
        });

        Ok(())
    }


    pub fn message_cluster_remove_cdn_node(
        &mut self,
        cluster_id: ClusterId,
        cdn_node_key: CdnNodeKey,
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut cdn_node: CdnNode = self.cdn_nodes.get(cdn_node_key)?;
        let mut cluster = self.clusters.get(cluster_id)?;

        if !cluster.only_manager(caller).is_ok()
            && !cdn_node.only_provider(caller).is_ok() {
                return Err(OnlyClusterManagerOrCdnNodeProvider);
        }

        cdn_node.only_with_cluster(cluster_id)?;
        
        cdn_node.unset_cluster();
        self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

        cluster.remove_cdn_node(cdn_node_key);
        self.clusters.update(cluster_id, &cluster)?;

        Self::env().emit_event(ClusterCdnNodeRemoved { 
            cluster_id, 
            cdn_node_key 
        });

        Ok(())
    }


    pub fn message_cluster_remove(
        &mut self,
        cluster_id: ClusterId, 
    ) -> Result<()> {
        let caller = Self::env().caller();

        let cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;
        cluster.only_without_nodes()?;
        
        self.clusters.remove(cluster_id);
        self.topology.remove_topology(cluster_id)?;

        Self::env().emit_event(ClusterRemoved { 
            cluster_id, 
        });

        Ok(())
    }


    pub fn message_cluster_set_node_status(
        &mut self, 
        cluster_id: ClusterId,
        node_key: NodeKey, 
        status_in_cluster: NodeStatusInCluster
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut node = self.nodes.get(node_key)?;
        let cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;

        node.change_status_in_cluster(status_in_cluster.clone());
        self.nodes.update(node_key, &node)?;

        Self::env().emit_event(ClusterNodeStatusSet { 
            node_key,
            cluster_id,
            status: status_in_cluster
        });
        
        Ok(())
    }


    pub fn message_cluster_set_cdn_node_status(
        &mut self, 
        cluster_id: ClusterId,
        cdn_node_key: CdnNodeKey, 
        status_in_cluster: NodeStatusInCluster
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut cdn_node = self.cdn_nodes.get(cdn_node_key)?;
        let cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;

        cdn_node.change_status_in_cluster(status_in_cluster.clone());
        self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

        Self::env().emit_event(ClusterCdnNodeStatusSet { 
            cdn_node_key,
            cluster_id,
            status: status_in_cluster
        });
        
        Ok(())
    }


    pub fn message_grant_trusted_manager_permission(
        &mut self,
        manager_id: AccountId
    ) -> Result<()> {
        let grantor = Self::env().caller();
        let permission = Permission::ClusterManagerTrustedBy(grantor);
        self.grant_permission(manager_id, permission)?;

        Self::env().emit_event(PermissionGranted { 
            account_id: manager_id,
            permission
        });
        
        Ok(())
    }


    pub fn message_revoke_trusted_manager_permission(
        &mut self,
        manager_id: AccountId
    ) -> Result<()> {
        let grantor = Self::env().caller();
        let permission = Permission::ClusterManagerTrustedBy(grantor);
        self.revoke_permission(manager_id, permission)?;

        Self::env().emit_event(PermissionRevoked { 
            account_id: manager_id,
            permission
        });

        Ok(())
    }


    pub fn message_cluster_set_params(
        &mut self,
        cluster_id: ClusterId,
        cluster_params: ClusterParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let mut cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;
        cluster.set_params(cluster_params.clone())?;
        self.clusters.update(cluster_id, &cluster)?;

        Self::env().emit_event(ClusterParamsSet { 
            cluster_id,
            cluster_params
        });

        Ok(())
    }


    pub fn message_cluster_reserve_resource(
        &mut self,
        cluster_id: ClusterId,
        resource: Resource,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let mut cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;
        cluster.put_resource(resource);
        self.clusters.update(cluster_id, &cluster)?;

        let cluster_v_nodes = self.topology.get_v_nodes_by_cluster(cluster_id);
        for v_node in cluster_v_nodes {
            let node_key = self.topology.get_node_by_v_node(cluster_id, v_node)?;
            let mut node = self.nodes.get(node_key)?;
            node.reserve_resource(resource)?;
            self.nodes.update(node_key, &node)?;
        }

        Self::env().emit_event(ClusterReserveResource {
            cluster_id,
            resource,
        });
        
        Ok(())
    }


    pub fn message_cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterInfo> {
        let cluster = self.clusters.get(cluster_id)?;
        let cluster_v_nodes = self.topology.get_v_nodes_by_cluster(cluster_id);

        Ok(ClusterInfo {
            cluster_id,
            cluster,
            cluster_v_nodes
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

            let cluster_v_nodes = self.topology.get_v_nodes_by_cluster(cluster_id);

            // Include the complete status of matched items.
            let cluster_info = ClusterInfo {
                cluster_id,
                cluster,
                cluster_v_nodes
            };

            clusters.push(cluster_info);
        }
        (clusters, self.clusters.next_cluster_id)
    }


    pub fn message_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) -> Result<()> {
        let mut cluster = self.clusters.get(cluster_id)?;

        // Charge the network fee from the cluster.
        self.capture_network_fee(&mut cluster.revenues)?;

        // Charge the cluster management fee.
        self.capture_fee(
            self.protocol.cluster_management_fee_bp(),
            cluster.manager_id,
            &mut cluster.revenues,
        )?;

        // Charge the provider payments from the cluster.
        let num_shares = cluster.nodes_keys.len() as Balance;
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


    // Set the price usd per gb
    pub fn message_cdn_set_rate(
        &mut self, 
        cluster_id: ClusterId, 
        cdn_usd_per_gb: Balance
    ) -> Result<()> {
        let caller = Self::env().caller();

        let mut cluster = self.clusters.get(cluster_id)?;
        cluster.only_manager(caller)?;
        cluster.cdn_set_rate(cdn_usd_per_gb);
        self.clusters.update(cluster_id, &cluster)?;

        Ok(())
    }


    // Get the price usd per gb
    pub fn message_cdn_get_rate(&self, cluster_id: ClusterId) -> Result<Balance> {
        let cluster = self.clusters.get(cluster_id)?;
        let rate = cluster.cdn_get_rate();
        Ok(rate)
    }


    // First payment is for aggregate consumption for account, second is the aggregate payment for the node (u32 for ids)
    pub fn message_cluster_put_cdn_revenue(
        &mut self, 
        cluster_id: ClusterId, 
        aggregates_accounts: Vec<(AccountId, u128)>, 
        aggregates_nodes: Vec<(CdnNodeKey, u128)>, 
        aggregates_buckets: Vec<(BucketId, Resource)>, 
        era: u64
    ) -> Result<()> {
        let mut cluster = self.clusters.get(cluster_id)?;

        let mut cluster_payment = 0;
        let mut _undistributed_payment_accounts = 0;

        let aggregate_payments_accounts;
        {
            let conv = &self.protocol.curr_converter;
            aggregate_payments_accounts = aggregates_accounts.iter().map(|(client_id, resources_used)| {
                let account_id = *client_id;
                let cere_payment: Balance = conv.to_cere(*resources_used as Balance * cluster.cdn_usd_per_gb / KB_PER_GB );
                (account_id, cere_payment)
            }).collect::<Vec<(AccountId, Balance)>>();
        }

        for &(client_id, payment) in aggregate_payments_accounts.iter() {
            if let Ok(mut account) = self.accounts.get(&client_id) {
                account.withdraw_bonded(Payable(payment))?;
                _undistributed_payment_accounts += payment;
                self.accounts.save(&client_id, &account);
            } else {
                return Err(InsufficientBalance)
            }
        };

        let conv = self.protocol.curr_converter.clone();
        let committer = &mut self.committer;

        for &(cdn_node_key, resources_used) in aggregates_nodes.iter() {
            let mut cdn_node = self.cdn_nodes.get(cdn_node_key)?;
            let protocol_fee = self.protocol.get_protocol_fee_bp();
            let protocol = &mut self.protocol;
            
            let payment = conv.to_cere(resources_used as Balance * cluster.cdn_usd_per_gb / KB_PER_GB );

            // let protocol_payment = payment * protocol_fee as u128/ BASIS_POINTS;
            let node_payment = payment * (BASIS_POINTS - protocol_fee) as u128 / BASIS_POINTS;
            let protocol_payment = payment - node_payment;
            
            cdn_node.put_payment(node_payment);

            protocol.put_revenues(Cash(protocol_payment));
            self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

            committer.set_validated_commit(cdn_node_key, era).unwrap();
            cluster_payment += node_payment;
        }
        // Add check that two payments should equal?

        // Go through buckets and deduct used resources
        for &(bucket_id, resources_used) in aggregates_buckets.iter() {
            let mut bucket = self.buckets.get(bucket_id)?;

            if bucket.resource_consumption_cap <= resources_used {
                bucket.resource_consumption_cap -= resources_used;
                self.buckets.update(bucket_id, &bucket)?;
            }
        }

        // Add revenues to cluster
        cluster.cdn_put_revenues(Cash(cluster_payment));
        self.clusters.update(cluster_id, &cluster)?;

        Ok(())
    }


    pub fn message_cluster_distribute_cdn_revenue(&mut self, cluster_id: ClusterId) -> Result<()> {
        let mut cluster = self.clusters.get(cluster_id)?;

        // Charge the network fee from the cluster.
        self.capture_network_fee(&mut cluster.cdn_revenues)?;

        // Charge the cluster management fee.
        self.capture_fee(
            self.protocol.cluster_management_fee_bp(),
            cluster.manager_id,
            &mut cluster.cdn_revenues
        )?;

        // First accumulated revenues to distribute.
        let mut distributed_revenue = 0;
    
        for cdn_node_key in &cluster.cdn_nodes_keys {
            let cdn_node = self.cdn_nodes.get(*cdn_node_key)?;
            distributed_revenue += cdn_node.undistributed_payment;
        }

        // Charge the provider payments from the cluster.
        cluster.cdn_revenues.pay(Payable(distributed_revenue))?;

        // Distribute revenues to nodes
        for cdn_node_key in &cluster.cdn_nodes_keys {
            let mut cdn_node = self.cdn_nodes.get(*cdn_node_key)?;

            Self::send_cash(cdn_node.provider_id, Cash(cdn_node.undistributed_payment))?;
            cdn_node.take_payment(cdn_node.undistributed_payment)?;
            self.cdn_nodes.update(*cdn_node_key, &cdn_node)?;

            Self::env().emit_event(ClusterDistributeCdnRevenues {
                cluster_id, 
                provider_id: cdn_node.provider_id 
            });
        }
        self.clusters.update(cluster_id, &cluster)?;

        Ok(())
    }


}
