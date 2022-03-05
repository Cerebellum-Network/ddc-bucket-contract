use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    vnode::entity::VNodeId,
};
use crate::ddc_bucket::flow::Flow;

pub type DealId = u32;

#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Deal {
    pub vnode_id: VNodeId,
    pub flow: Flow,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct DealStatus {
    pub vnode_id: VNodeId,
    pub estimated_rent_end_ms: u64,
}
