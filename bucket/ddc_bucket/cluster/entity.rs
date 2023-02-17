//! The data structure of Clusters.
// use ink_storage::Mapping;
// use ink_prelude::vec::Vec;
use ink_prelude::vec::Vec;
use ink_storage::collections::HashMap;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::Error::UnauthorizedClusterManager;
use crate::ddc_bucket::{AccountId, Balance, Error::InsufficientResources, Result};

pub type ClusterId = u32;
pub type ClusterParams = Params;
pub type VNodeIndex = u32;
pub type VNodeId = (ClusterId, VNodeIndex);

#[derive(Clone, PartialEq, Encode, Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub manager_id: AccountId,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub v_nodes: Vec<u64>,
    pub total_rent: Balance,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct ClusterStatus {
    pub cluster_id: ClusterId,
    pub cluster: Cluster,
    pub params: Params,
}

impl Cluster {
    pub fn new(manager_id: AccountId, v_nodes: Vec<u64>) -> Self {
        Cluster {
            manager_id,
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            v_nodes,
            total_rent: 0,
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

    pub fn change_rent(&mut self, rent: Balance) {
        self.total_rent = rent;
    }
}
