//! This module captures fees on behalf of the entire Cere network.
use scale::{Decode, Encode};
use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::perm::entity::Permission;

pub type BasisPoints = Balance;

const BP: BasisPoints = 10_000; // 100%

#[ink::storage_item]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NetworkFeeStore(
    pub FeeConfig,
);

impl NetworkFeeStore {

    pub fn new() -> Self {
        Self(FeeConfig::new())
    }

    pub fn cluster_management_fee_bp(&self) -> BasisPoints {
        self.0.cluster_management_fee_bp
    }
}

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout, Debug))]
pub struct FeeConfig {
    /// The fee rate from cluster revenues to the overall network. In basis points (1% of 1%).
    pub network_fee_bp: BasisPoints,
    /// The destination account of network fees. Use the 0 account to burn the fees.
    pub network_fee_destination: AccountId,

    /// The fee rate from cluster revenues to the cluster manager. In basis points (1% of 1%).
    pub cluster_management_fee_bp: BasisPoints,
}

impl FeeConfig {
    pub fn new() -> Self {
        FeeConfig {
            network_fee_bp: BasisPoints::default(),
            // todo: must be revised due to https://use.ink/faq/migrating-from-ink-3-to-4#removal-of-accountid-default-implementation
            network_fee_destination: AccountId::from([0x00; 32]),
            cluster_management_fee_bp: BasisPoints::default(),
        }
    }
}

impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(store: &NetworkFeeStore, revenues: &mut Cash) -> Result<()> {
        let config = &store.0;
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
        self.network_fee.0 = config;
        Ok(())
    }
}
