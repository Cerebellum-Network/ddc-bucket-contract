//! This module captures fees on behalf of the entire Cere network.

use ink_storage::{Lazy, traits};

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::perm::entity::Permission;

const BP: Balance = 10_000; // 100%

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NetworkFeeStore(
    pub Lazy<Balance>,
);


impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(store: &NetworkFeeStore, revenues: &mut Cash) -> Result<()> {
        let network_fee = revenues.peek() * (*store.0) / BP;
        let (payable, cash) = Cash::borrow_payable_cash(network_fee);
        revenues.pay(payable)?;
        let to_burn = AccountId::default();
        Self::send_cash(to_burn, cash)
    }

    pub fn message_admin_set_network_fee(&mut self, network_fee_bp: Balance) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        *self.network_fee.0 = network_fee_bp;
        Ok(())
    }
}
