use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, Deposit, FlowId, InsufficientBalance, Payable, Result, Schedule};

impl DdcBucket {
    pub fn message_deposit(&mut self) -> Result<()> {
        // Receive the payable value.
        let cash = Self::receive_cash();
        let value = cash.peek();
        let account_id = Self::env().caller();

        self.accounts.deposit(account_id, cash);
        Self::env().emit_event(Deposit { account_id, value });
        Ok(())
    }

    pub fn billing_withdraw(&mut self, from: AccountId, payable: Payable) -> Result<()> {
        let account = self.accounts.0.get_mut(&from)
            .ok_or(InsufficientBalance)?;

        let time_ms = Self::env().block_timestamp();
        account.withdraw(time_ms, payable)?;
        Ok(())
    }

    pub fn billing_get_net(&self, from: AccountId) -> Balance {
        match self.accounts.0.get(&from) {
            None => 0,
            Some(account) => {
                let time_ms = Self::env().block_timestamp();
                account.get_withdrawable(time_ms)
            }
        }
    }

    pub fn billing_transfer(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
        let (payable, cash) = Cash::borrow_payable_cash(amount);
        self.billing_withdraw(from, payable)?;
        self.accounts.deposit(to, cash);
        Ok(())
    }

    pub fn billing_start_flow(&mut self, from: AccountId, rate: Balance) -> Result<FlowId> {
        let start_ms = Self::env().block_timestamp();
        let cash_schedule = Schedule::new(start_ms, rate);
        let payable_schedule = cash_schedule.clone();

        let from_account = self.accounts.get_mut(&from)?;
        from_account.lock_schedule(payable_schedule);

        let flow_id = self.flows.create(from, cash_schedule);
        Ok(flow_id)
    }

    pub fn billing_flow_covered_until(&self, flow_id: FlowId) -> Result<u64> {
        let flow = self.flows.get(flow_id)?;
        let account = self.accounts.get(&flow.from)?;
        Ok(account.schedule_covered_until())
    }

    pub fn billing_settle_flow(&mut self, flow_id: FlowId) -> Result<Cash> {
        let now_ms = Self::env().block_timestamp();

        let flow = self.flows.get_mut(flow_id)?;
        let flowed_amount = flow.schedule.take_value_at_time(now_ms);
        let (payable, cash) = Cash::borrow_payable_cash(flowed_amount);

        let account = self.accounts.get_mut(&flow.from)?;
        account.pay_scheduled(now_ms, payable)?;
        Ok(cash)
    }

    pub fn receive_cash() -> Cash {
        Cash(Self::env().transferred_balance())
    }

    pub fn send_cash(destination: AccountId, cash: Cash) -> Result<()> {
        match Self::env().transfer(destination, cash.consume()) {
            Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
            Ok(_v) => Ok(()),
        }
    }
}
