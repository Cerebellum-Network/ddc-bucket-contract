//! The data structure of Clusters.
// use ink_storage::Mapping;
// use ink_prelude::vec::Vec;
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, PackedLayout, PackedAllocate};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::node::entity::{Resource, NodeKey};
use crate::ddc_bucket::cdn_node::entity::{CdnNodeKey};

use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::topology::store::VNodeToken;
use crate::ddc_bucket::Error::UnauthorizedClusterManager;
use crate::ddc_bucket::{AccountId, Balance, Error::InsufficientResources, Result};

pub type ClusterId = u32;
pub type ClusterParams = Params;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub manager_id: AccountId,
    pub cluster_params: ClusterParams,

    // storage nodes
    pub nodes_keys: Vec<NodeKey>,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub total_rent: Balance,

    // cdn nodes
    pub cdn_nodes_keys: Vec<CdnNodeKey>,
    pub cdn_resources_used: Resource,
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
}

impl Cluster {

    pub fn new(
        manager_id: AccountId,
        cluster_params: ClusterParams,
    ) -> Self {
        Cluster {
            manager_id,
            cluster_params,
            nodes_keys: Vec::new(),
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            total_rent: 0,
            cdn_nodes_keys: Vec::new(),
            cdn_usd_per_gb: 104_857_600, // setting initially to 1 cent per GB
            cdn_resources_used: 0,
            cdn_revenues: Cash(0),
        }
    }

    pub fn get_rent(&self, resource: Resource) -> Balance {
        return self.total_rent * resource as Balance;
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.resource_per_vnode += amount;
    }

    pub fn take_resource(&mut self, amount: Resource) -> Result<()> {
        let used = self.resource_used + amount;
        if used > self.resource_per_vnode {
            return Err(InsufficientResources);
        }
        self.resource_used = used;
        Ok(())
    }

    pub fn only_manager(&self, caller: AccountId) -> Result<()> {
        if self.manager_id == caller {
            Ok(())
        } else {
            Err(UnauthorizedClusterManager)
        }
    }

    pub fn set_rent(&mut self, rent: Balance) {
        self.total_rent = rent;
    }
}
