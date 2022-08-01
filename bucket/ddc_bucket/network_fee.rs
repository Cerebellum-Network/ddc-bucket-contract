//! This module captures fees on behalf of the entire Cere network.

use ink_storage::{Lazy, traits};

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::perm::entity::Permission;

const BP: Balance = 10_000; // 100%

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NetworkFeeStore(
    pub Lazy<(Balance, AccountId)>,
);


impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(store: &NetworkFeeStore, revenues: &mut Cash) -> Result<()> {
        let (rate_bp, destination) = *store.0;
        let network_fee = revenues.peek() * rate_bp / BP;
        let (payable, cash) = Cash::borrow_payable_cash(network_fee);
        revenues.pay(payable)?;
        Self::send_cash(destination, cash)
    }

    pub fn message_admin_set_network_fee(&mut self, rate_bp: Balance, destination: AccountId) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        *self.network_fee.0 = (rate_bp, destination);
        Ok(())
    }
}
