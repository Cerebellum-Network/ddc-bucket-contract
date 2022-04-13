//! The data structure that represents the possible permissions an account may have.

use scale::{Decode, Encode};

use crate::ddc_bucket::AccountId;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub enum Permission {
    ManagerTrustedBy(AccountId),
    SetExchangeRate,
    SuperAdmin,
}
