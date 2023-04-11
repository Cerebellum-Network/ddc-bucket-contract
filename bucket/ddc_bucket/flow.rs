//! The Flow data structure represents an outgoing stream of payments from an account.
use scale::{Decode, Encode};
use crate::ddc_bucket::{
    AccountId,
    schedule::Schedule,
};

// TODO: remove Clone.
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout, Debug))]
pub struct Flow {
    pub from: AccountId,
    pub schedule: Schedule,
}
