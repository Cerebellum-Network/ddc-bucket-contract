//! The data structure of Clusters.

use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::InsufficientResources, NodeId, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_INDEX, SIZE_PER_RECORD, SIZE_RESOURCE, SIZE_VEC};
use crate::ddc_bucket::Error::UnauthorizedClusterManager;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::params::store::Params;

pub type ClusterId = u32;
pub type ClusterParams = Params;
pub type PartitionIndex = u32;
pub type PartitionId = (ClusterId, PartitionIndex);

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub cluster_id: ClusterId,
    pub manager_id: AccountId,
    pub vnodes: Vec<NodeId>,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub total_rent: Balance,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct ClusterStatus {
    pub cluster: Cluster,
    pub params: Params,
}

impl Cluster {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_INDEX
            + SIZE_ACCOUNT_ID
            + SIZE_VEC + self.vnodes.len() * SIZE_INDEX
            + SIZE_RESOURCE
            + SIZE_RESOURCE
            + SIZE_BALANCE
            + SIZE_BALANCE
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
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
        if self.manager_id == caller { Ok(()) } else { Err(UnauthorizedClusterManager) }
    }
}