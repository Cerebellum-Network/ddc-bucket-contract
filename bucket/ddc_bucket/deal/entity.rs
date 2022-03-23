use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    contract_fee::{SIZE_INDEX, SIZE_PER_RECORD},
    flow::Flow,
    node::entity::NodeId,
};

pub type DealId = u32;

#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Deal {
    pub node_id: NodeId,
    pub flow: Flow,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct DealStatus {
    pub node_id: NodeId,
    pub estimated_rent_end_ms: u64,
}

impl Deal {
    pub const RECORD_SIZE: usize = SIZE_PER_RECORD + SIZE_INDEX + 8;
}
