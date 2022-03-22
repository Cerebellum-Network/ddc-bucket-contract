use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{
    account::entity::Account,
    AccountId, Balance, Cash,
    contract_fee::calculate_contract_fee,
    DdcBucket, Deposit, InsufficientBalance, Payable, Result,
};

impl DdcBucket {
    pub fn message_deposit(&mut self) -> Result<()> {
        // Receive the payable value, minus the contract fee.
        let mut cash = Self::receive_cash();
        cash.pay(calculate_contract_fee(Account::RECORD_SIZE))?;
        let value = cash.peek();

        let account_id = Self::env().caller();
        self.accounts.deposit(account_id, cash);
        Self::env().emit_event(Deposit { account_id, value });
        Ok(())
    }

    pub fn receive_cash() -> Cash {
        Cash(Self::env().transferred_balance())
    }

    pub fn send_cash(destination: AccountId, cash: Cash) -> Result<()> {
        if cash.peek() == 0 { return Ok(()); }
        match Self::env().transfer(destination, cash.consume()) {
            Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
            Ok(_v) => Ok(()),
        }
    }


    fn _account_withdraw(&mut self, from: AccountId, payable: Payable) -> Result<()> {
        let account = self.accounts.0.get_mut(&from)
            .ok_or(InsufficientBalance)?;

        let time_ms = Self::env().block_timestamp();
        account.withdraw(time_ms, payable)?;
        Ok(())
    }

    fn _account_get_net(&self, from: AccountId) -> Balance {
        match self.accounts.0.get(&from) {
            None => 0,
            Some(account) => {
                let time_ms = Self::env().block_timestamp();
                account.get_withdrawable(time_ms)
            }
        }
    }

    fn _account_transfer(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
        let (payable, cash) = Cash::borrow_payable_cash(amount);
        self._account_withdraw(from, payable)?;
        self.accounts.deposit(to, cash);
        Ok(())
    }
}
