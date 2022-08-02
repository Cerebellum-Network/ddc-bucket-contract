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
        let network_fee = revenues.peek() * config.network_fee_bp / BP;
        let (payable, cash) = Cash::borrow_payable_cash(network_fee);
        revenues.pay(payable)?;
        Self::send_cash(config.network_fee_destination, cash)
    }

    pub fn message_admin_set_fee_config(&mut self, config: FeeConfig) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        *self.network_fee.0 = config;
        Ok(())
    }
}
