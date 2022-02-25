use ink_prelude::{
    vec, vec::Vec,
};
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    collections::Stash,
    collections::Vec as InkVec,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::billing_account::{BillingAccount};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct AccountStore(pub HashMap<AccountId, BillingAccount>);

impl AccountStore {

}
