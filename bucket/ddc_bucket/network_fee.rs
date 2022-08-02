//! This module captures fees on behalf of the entire Cere network.

use ink_storage::{Lazy, traits};
use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::perm::entity::Permission;

pub type BasisPoints = Balance;

const BP: BasisPoints = 10_000; // 100%

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NetworkFeeStore(
    pub Lazy<FeeConfig>,
);

impl NetworkFeeStore {
    pub fn cluster_management_fee_bp(&self) -> BasisPoints {
        self.0.cluster_management_fee_bp
    }
}

#[derive(traits::SpreadLayout, Default, Decode, Encode, TypeInfo)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct FeeConfig {
    pub network_fee_bp: BasisPoints,
    pub network_fee_destination: AccountId,

    pub cluster_management_fee_bp: BasisPoints,
}


impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(store: &NetworkFeeStore, revenues: &mut Cash) -> Result<()> {
        let config = &*store.0;
        Self::capture_fee(config.network_fee_bp, config.network_fee_destination, revenues)
    }

    /// Take a fee from the given revenues (in place) and send it to the destination.
    pub fn capture_fee(rate_bp: Balance, destination: AccountId, revenues: &mut Cash) -> Result<()> {
        let fee = revenues.peek() * rate_bp / BP;
        let (payable, cash) = Cash::borrow_payable_cash(fee);
        revenues.pay(payable)?;
        Self::send_cash(destination, cash)
    }

    pub fn message_admin_set_fee_config(&mut self, config: FeeConfig) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        *self.network_fee.0 = config;
        Ok(())
    }
}
