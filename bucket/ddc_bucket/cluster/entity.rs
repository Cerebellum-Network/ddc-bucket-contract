use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::contract_fee::{SIZE_INDEX, SIZE_PER_RECORD, SIZE_VEC};
use crate::ddc_bucket::VNodeId;

pub type ClusterId = u32;
pub type ClusterParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub cluster_id: ClusterId,
    pub cluster_params: ClusterParams,
    pub vnode_ids: Vec<VNodeId>,
}

impl Cluster {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_INDEX + SIZE_VEC + SIZE_VEC
            + self.cluster_params.len()
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }
}