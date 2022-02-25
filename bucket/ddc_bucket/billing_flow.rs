use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::AccountId;

use super::schedule::Schedule;

pub type FlowId = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BillingFlow {
    pub from: AccountId,
    pub schedule: Schedule,
}