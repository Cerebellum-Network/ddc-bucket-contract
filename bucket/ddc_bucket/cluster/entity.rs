//! The data structure of Clusters.
use ink_prelude::vec::Vec;
use ink_prelude::string::String;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, PackedLayout, PackedAllocate};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::node::entity::{Resource, NodeKey};
use crate::ddc_bucket::cdn_node::entity::{CdnNodeKey};
use crate::ddc_bucket::Error::{OnlyClusterManager, InsufficientBalance};
use crate::ddc_bucket::{AccountId, Balance, VNodeToken, Result, Error::*};


pub type ClusterId = u32;
pub type ClusterParams = String;

// https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout
// There is a buffer with only limited capacity (around 16KB in the default configuration) available.
pub const MAX_CLUSTER_NODES_LEN_IN_VEC: usize = 200;
pub const MAX_CLUSTER_CDN_NODES_LEN_IN_VEC: usize = 200;


#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub manager_id: AccountId,
    pub cluster_params: ClusterParams,

    // storage nodes
    pub nodes_keys: Vec<NodeKey>,
    pub resource_per_v_node: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub total_rent: Balance,

    // cdn nodes
    pub cdn_nodes_keys: Vec<CdnNodeKey>,
    pub cdn_revenues: Cash,
    pub cdn_usd_per_gb: Balance,
}

// https://use.ink/3.x/ink-vs-solidity#nested-mappings--custom--advanced-structures
#[allow(unconditional_recursion)]
impl ink_storage::traits::PackedAllocate for Cluster {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct ClusterInfo {
    pub cluster_id: ClusterId,
    pub cluster: Cluster,
    pub cluster_v_nodes: Vec<VNodeToken>,
}

pub const CLUSTER_PARAMS_MAX_LEN: usize = 100_000;
pub const CDN_USD_PER_GB : Balance = 104_857_600;
pub const KB_PER_GB: Balance = 1_000_000;

impl Cluster {

    pub fn new(
        manager_id: AccountId,
        cluster_params: ClusterParams,
        resource_per_v_node: Resource,
    ) -> Result<Self> {

        let mut cluster = Cluster {
            manager_id,
            cluster_params: ClusterParams::default(),
            nodes_keys: Vec::new(),
            resource_per_v_node,
            resource_used: 0,
            revenues: Cash(0),
            total_rent: 0,
            cdn_nodes_keys: Vec::new(),
            cdn_usd_per_gb: CDN_USD_PER_GB, // setting initially to 1 cent per GB
            cdn_revenues: Cash(0),
        };
        
        cluster.set_params(cluster_params)?;
        Ok(cluster)
    }

    pub fn only_manager(&self, caller: AccountId) -> Result<()> {
        (self.manager_id == caller)
            .then(|| ())
            .ok_or(OnlyClusterManager)
    }

    pub fn only_without_nodes(&self) -> Result<()> {
        if self.nodes_keys.is_empty() && self.cdn_nodes_keys.is_empty() {
            Ok(())
        } else {
            Err(ClusterIsNotEmpty)
        }
    }

    pub fn add_node(&mut self, node_key: NodeKey) -> Result<()> {
        if self.nodes_keys.len() + 1 > MAX_CLUSTER_NODES_LEN_IN_VEC {
            return Err(NodesSizeExceedsLimit);
        }
        self.nodes_keys.push(node_key);
        Ok(())
    }

    pub fn remove_node(&mut self, node_key: NodeKey) {
        if let Some(pos) = self.nodes_keys.iter().position(|x| *x == node_key) {
            self.nodes_keys.remove(pos);
        }
    }

    pub fn add_cdn_node(&mut self, cdn_node_key: CdnNodeKey) -> Result<()> {
        if self.cdn_nodes_keys.len() + 1 > MAX_CLUSTER_CDN_NODES_LEN_IN_VEC {
            return Err(CdnNodesSizeExceedsLimit);
        }
        self.cdn_nodes_keys.push(cdn_node_key);
        Ok(())
    }

    pub fn remove_cdn_node(&mut self, cdn_node_key: CdnNodeKey) {
        if let Some(pos) = self.cdn_nodes_keys.iter().position(|x| *x == cdn_node_key) {
            self.cdn_nodes_keys.remove(pos);
        }    
    }

    pub fn set_params(&mut self, cluster_params: ClusterParams) -> Result<()> {
        if cluster_params.len() > CLUSTER_PARAMS_MAX_LEN {
            return Err(ParamsSizeExceedsLimit);
        }
        self.cluster_params = cluster_params;
        Ok(())
    }

    pub fn get_rent(&self, resource: Resource) -> Balance {
        let rent = self.total_rent * resource as Balance;
        rent
    }

    pub fn set_resource_per_v_node(&mut self, resource_per_v_node: Resource) {
        self.resource_per_v_node = resource_per_v_node;
    }

    pub fn take_resource(&mut self, amount: Resource) -> Result<()> {
        let used = self.resource_used + amount;
        if used > self.resource_per_v_node {
            return Err(InsufficientResources);
        }
        self.resource_used = used;
        Ok(())
    }

    pub fn cdn_get_revenue_cere(&self) -> Cash {
        self.cdn_revenues
    }

    pub fn cdn_set_rate(&mut self, cdn_usd_per_gb: Balance) {
        self.cdn_usd_per_gb = cdn_usd_per_gb;
    }

    pub fn cdn_get_rate(&self) -> Balance {
        self.cdn_usd_per_gb
    }

    pub fn cdn_put_revenues(&mut self, amount: Cash) {
        self.cdn_revenues.increase(amount);
    }

    pub fn cdn_take_revenues(&mut self, amount: Payable) -> Result<()> {
        if amount.peek() > self.cdn_revenues.peek() {
            return Err(InsufficientBalance);
        }
        self.cdn_revenues.pay_unchecked(amount);
        Ok(())
    }

}
