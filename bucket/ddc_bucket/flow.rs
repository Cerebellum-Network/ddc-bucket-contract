use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    AccountId,
    schedule::Schedule,
};

#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Flow {
    pub from: AccountId,
    pub schedule: Schedule,
}
