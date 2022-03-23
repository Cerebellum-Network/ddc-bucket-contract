use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, NodeId};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_INDEX, SIZE_PER_RECORD, SIZE_VEC};

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
}

impl Cluster {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_INDEX
            + SIZE_ACCOUNT_ID
            + SIZE_VEC + self.cluster_params.len()
            + SIZE_VEC + self.vnodes.len() * SIZE_INDEX
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }
}