//! The Flow data structure represents an outgoing stream of payments from an account.

use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{schedule::Schedule, AccountId};

// TODO: remove Clone.
#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Flow {
    pub from: AccountId,
    pub schedule: Schedule,
}
