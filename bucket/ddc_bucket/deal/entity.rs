use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    flow::entity::FlowId,
    service::entity::VNodeId,
};

pub type DealId = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Deal {
    pub vnode_id: VNodeId,
    pub flow_id: FlowId,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct DealStatus {
    pub vnode_id: VNodeId,
    pub estimated_rent_end_ms: u64,
}
