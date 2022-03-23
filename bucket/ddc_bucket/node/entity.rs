use ink_prelude::{
    string::String,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_INDEX, SIZE_PER_RECORD, SIZE_VEC};

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type NodeParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Node {
    pub node_id: NodeId,
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub node_params: NodeParams,
}

impl Node {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_INDEX + SIZE_ACCOUNT_ID + SIZE_BALANCE
            + SIZE_VEC + self.node_params.len()
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }

    pub fn revenue_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
        if self.provider_id == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
    }
}