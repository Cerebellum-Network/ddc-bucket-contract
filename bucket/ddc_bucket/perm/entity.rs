//! The data structure that represents the possible permissions an account may have.

use ink_env::hash::{Blake2x256, HashOutput};
use ink_storage::{
    collections::HashMap,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};
use scale::{Decode, Encode};

use crate::ddc_bucket::AccountId;
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_HASHMAP, SIZE_PER_RECORD};

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub enum Perm {
    TrustedBy(AccountId),
}
