use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Error::InsufficientResources, NodeId, Result};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_INDEX, SIZE_PER_RECORD, SIZE_RESOURCE, SIZE_VEC};
use crate::ddc_bucket::node::entity::Resource;

pub type ClusterId = u32;
pub type ClusterParams = String;
pub type PartitionIndex = u32;
pub type PartitionId = (ClusterId, PartitionIndex);

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub cluster_id: ClusterId,
    pub manager: AccountId,
    pub cluster_params: ClusterParams,
    pub vnodes: Vec<NodeId>,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
}

impl Cluster {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_INDEX
            + SIZE_ACCOUNT_ID
            + SIZE_VEC + self.cluster_params.len()
            + SIZE_VEC + self.vnodes.len() * SIZE_INDEX
            + SIZE_RESOURCE + SIZE_RESOURCE
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
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
}