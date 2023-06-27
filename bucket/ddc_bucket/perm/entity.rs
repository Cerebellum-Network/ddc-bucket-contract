//! The data structure that represents the possible permissions an account may have.

use scale::{Decode, Encode};

use crate::ddc_bucket::AccountId;

#[derive(Copy, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
pub enum Permission {
    ClusterManagerTrustedBy(AccountId),
    SetExchangeRate,
    SuperAdmin,
}
