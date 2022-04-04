//! The Flow data structure represents an outgoing stream of payments from an account.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    AccountId,
    schedule::Schedule,
};
use crate::ddc_bucket::contract_fee::SIZE_ACCOUNT_ID;

// TODO: remove Clone.
#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Flow {
    pub from: AccountId,
    pub schedule: Schedule,
}

impl Flow {
    pub const RECORD_SIZE: usize = SIZE_ACCOUNT_ID + Schedule::RECORD_SIZE;
}
