use ink_prelude::string::String;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    flow::entity::FlowId,
    service::entity::ServiceId,
};

pub type DealId = u32;
pub type DealParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Deal {
    pub service_id: ServiceId,
    pub flow_id: FlowId,
    pub deal_params: DealParams,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct DealStatus {
    pub service_id: ServiceId,
    pub estimated_rent_end_ms: u64,
    pub deal_params: DealParams,
}
