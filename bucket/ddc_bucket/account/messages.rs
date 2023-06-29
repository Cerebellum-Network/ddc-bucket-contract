//! The public interface of Accounts and deposits.

use ink_prelude::vec::Vec;
use ink_lang::codegen::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, Deposit, Payable, Result, TOKEN};
use crate::ddc_bucket::Error::InsufficientBalance;
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {

    // todo: remove this method as we can not support iterable data structures of arbitrary data size
    pub fn message_get_accounts(&self) -> Vec<AccountId> {
        self.accounts.accounts_keys.iter().cloned().collect()
    }

    pub fn message_account_deposit(&mut self) -> Result<()> {
        // Receive the payable value, minus the contract fee.
        let cash = Self::receive_cash();
        let account_id = Self::env().caller();

        // Create the account, if necessary.
        self.accounts.create_if_not_exist(account_id);

        Self::env().emit_event(Deposit { account_id, value: cash.peek() });

        let mut account = self.accounts.get(&account_id)?;
        account.deposit(cash);
        self.accounts.save(&account_id, &account);

        Ok(())
    }

    pub fn message_account_bond(&mut self, bond_amount: Balance) -> Result<()> {
        let time_ms = Self::env().block_timestamp();
        let account_id = Self::env().caller();

        if let Ok(mut account) = self.accounts.get(&account_id) {
            let conv = &self.protocol.curr_converter;
            account.bond(time_ms, conv, bond_amount)?;
            self.accounts.save(&account_id, &account);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn message_account_unbond(&mut self, amount_to_unbond: Cash) -> Result<()> {
        let time_ms = Self::env().block_timestamp();
        let account_id = Self::env().caller();

        let mut account = self.accounts.get(&account_id)?;
        account.unbond(amount_to_unbond, time_ms)?;
        self.accounts.save(&account_id, &account);

        Ok(())
    }

    pub fn message_account_withdraw_unbonded(&mut self) -> Result<()> {
        let time_ms = Self::env().block_timestamp();
        let account_id = Self::env().caller();

        let mut account = self.accounts.get(&account_id)?;
        account.withdraw_unbonded(time_ms)?;
        self.accounts.save(&account_id, &account);

        Ok(())
    }

    pub fn message_account_get_usd_per_cere(&self) -> Balance {
        self.protocol.curr_converter.to_usd(1 * TOKEN)
    }

    pub fn message_account_set_usd_per_cere(&mut self, usd_per_cere: Balance) {
        self.only_with_permission(Permission::SetExchangeRate).unwrap();
        self.protocol.curr_converter.set_usd_per_cere(usd_per_cere)
    }

    pub fn receive_cash() -> Cash {
        Cash(Self::env().transferred_value())
    }

    pub fn send_cash(destination: AccountId, cash: Cash) -> Result<()> {
        if cash.peek() == 0 { return Ok(()); }
        match Self::env().transfer(destination, cash.consume()) {
            Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
            Ok(_v) => Ok(()),
        }
    }

    fn _account_withdraw(&mut self, from: AccountId, payable: Payable) -> Result<()> {
        if let Ok(mut account) = self.accounts.get(&from) {
            let time_ms = Self::env().block_timestamp();
            let conv = &self.protocol.curr_converter;
            account.withdraw(time_ms, conv, payable)?;
            self.accounts.save(&from, &account);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    fn _account_withdraw_bonded(&mut self, account_id: AccountId, payable: Payable) -> Result<()> {
        if let Ok(mut account) = self.accounts.get(&account_id) {
            account.withdraw_bonded(payable)?;
            self.accounts.save(&account_id, &account);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    fn _account_get_net(&self, from: AccountId) -> Balance {
        match self.accounts.accounts.get(&from) {
            None => 0,
            Some(account) => {
                let time_ms = Self::env().block_timestamp();
                let conv = &self.protocol.curr_converter;
                account.get_withdrawable(time_ms, conv)
            }
        }
    }

    fn _account_transfer(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
        let (payable, cash) = Cash::borrow_payable_cash(amount);
        self._account_withdraw(from, payable)?;

        // Create the account, if necessary.
        self.accounts.create_if_not_exist(to);

        let mut account = self.accounts.get(&to)?;
        account.deposit(cash);
        self.accounts.save(&to, &account);
        Ok(())
    }
}
