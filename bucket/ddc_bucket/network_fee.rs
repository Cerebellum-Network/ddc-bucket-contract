//! This module captures fees on behalf of the entire Cere network.

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::perm::entity::Permission;

pub type BasisPoints = Balance;

const BP: BasisPoints = 10_000; // 100%

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct NetworkFeeStore {
    pub fee_config: FeeConfig,
}

impl NetworkFeeStore {
    pub fn cluster_management_fee_bp(&self) -> BasisPoints {
        self.fee_config.cluster_management_fee_bp
    }
}

/// The configuration of fees.
#[derive(SpreadAllocate, SpreadLayout, Default, Decode, Encode)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug, scale_info::TypeInfo))]
pub struct FeeConfig {
    /// The fee rate from cluster revenues to the overall network. In basis points (1% of 1%).
    pub network_fee_bp: BasisPoints,
    /// The destination account of network fees. Use the 0 account to burn the fees.
    pub network_fee_destination: AccountId,
    /// The fee rate from cluster revenues to the cluster manager. In basis points (1% of 1%).
    pub cluster_management_fee_bp: BasisPoints,
}


impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(store: &NetworkFeeStore, revenues: &mut Cash) -> Result<()> {
        let config = &store.fee_config;
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
        self.network_fee.fee_config = config;
        Ok(())
    }
}
