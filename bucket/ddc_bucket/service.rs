use ink_prelude::{
    string::String,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

pub type ProviderId = AccountId;
pub type ServiceId = u32;
pub type ServiceParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Service {
    pub service_id: ServiceId,
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub service_params: ServiceParams,
}

impl Service {
    pub fn revenue_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
        if self.provider_id == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
    }
}