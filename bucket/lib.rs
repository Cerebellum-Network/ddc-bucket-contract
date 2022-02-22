#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_bucket {
    use ink_prelude::{
        string::String,
        vec, vec::Vec,
    };
    use ink_storage::{
        collections::{HashMap, hashmap::Entry::*},
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    use Error::*;

    #[ink(storage)]
    pub struct DdcBucket {
        buckets: Stash<Bucket>,
        providers: HashMap<AccountId, Provider>,

        billing_accounts: HashMap<AccountId, BillingAccount>,
        billing_flows: Stash<BillingFlow>,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                buckets: Stash::new(),
                providers: HashMap::new(),
                billing_accounts: HashMap::new(),
                billing_flows: Stash::new(),
            }
        }
    }


    // ---- Bucket ----
    pub type BucketId = u32;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Bucket {
        owner_id: AccountId,
        provider_id: AccountId,
        rent_per_month: Balance,
        flow_id: FlowId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketCreated {
        #[ink(topic)]
        bucket_id: BucketId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Deposit {
        #[ink(topic)]
        account_id: AccountId,
        value: Balance,
    }

    #[derive(Clone, PartialEq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct BucketStatus {
        provider_id: AccountId,
        estimated_rent_end_ms: u64,
        writer_ids: Vec<AccountId>,
    }

    impl DdcBucket {
        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_create(&mut self, provider_id: AccountId) -> Result<BucketId> {
            // Receive the payable value.
            self.deposit()?;
            let caller = Self::env().caller();

            // Start the payment flow for a bucket.
            let rent_per_month = self.get_provider_rent(provider_id)?;
            let flow_id = self.billing_start_flow(caller, provider_id, rent_per_month)?;

            // Create a new bucket.
            let bucket = Bucket {
                owner_id: caller,
                provider_id,
                rent_per_month,
                flow_id,
            };
            let bucket_id = self.buckets.put(bucket);

            Self::env().emit_event(BucketCreated { bucket_id });
            Ok(bucket_id)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self) -> Result<()> {
            // Receive the payable value.
            let cash = Self::receive_cash();
            let value = cash.0;
            let account_id = Self::env().caller();
            self.billing_deposit(account_id, cash);

            Self::env().emit_event(Deposit { account_id, value });
            Ok(())
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            let bucket = self.buckets.get(bucket_id)
                .ok_or(Error::BucketDoesNotExist)?;

            let estimated_rent_end_ms = self.billing_flow_covered_until(bucket.flow_id)?;

            Ok(BucketStatus {
                provider_id: bucket.provider_id,
                estimated_rent_end_ms,
                writer_ids: vec![bucket.owner_id],
            })
        }

        fn get_provider_rent(&self, provider_id: AccountId) -> Result<Balance> {
            let provider = self.providers.get(&provider_id)
                .ok_or(Error::ProviderDoesNotExist)?;
            Ok(provider.rent_per_month)
        }
    }
    // ---- End Bucket ----


    // ---- Provider ----
    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Provider {
        rent_per_month: Balance,
        location: String,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ProviderSetInfo {
        #[ink(topic)]
        provider_id: AccountId,
        rent_per_month: Balance,
        location: String,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ProviderWithdraw {
        #[ink(topic)]
        provider_id: AccountId,
        #[ink(topic)]
        bucket_id: BucketId,
        value: Balance,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn provider_set_info(&mut self, rent_per_month: Balance, location: String) -> Result<()> {
            let provider_id = self.env().caller();
            self.providers.insert(provider_id, Provider {
                rent_per_month,
                location: location.clone(),
            });

            Self::env().emit_event(ProviderSetInfo { provider_id, rent_per_month, location });
            Ok(())
        }

        #[ink(message)]
        pub fn provider_get_info(&self, provider_id: AccountId) -> Result<Provider> {
            self.providers.get(&provider_id)
                .cloned()
                .ok_or(Error::ProviderDoesNotExist)
        }

        #[ink(message)]
        pub fn provider_withdraw(&mut self, bucket_id: BucketId) -> Result<()> {
            let provider_id = self.env().caller();

            let flow_id = {
                let bucket = self.buckets.get(bucket_id)
                    .ok_or(Error::BucketDoesNotExist)?;
                if bucket.provider_id != provider_id {
                    return Err(Error::UnauthorizedProvider);
                }
                bucket.flow_id
            };

            let flowed_amount = self.billing_settle_flow(flow_id)?;

            let (payable, cash) = Self::borrow_payable_cash(flowed_amount);
            self.billing_withdraw(provider_id, payable)?;
            Self::send_cash(provider_id, cash)?;

            Self::env().emit_event(ProviderWithdraw { provider_id, bucket_id, value: flowed_amount });
            Ok(())
        }
    }
    // ---- End Provider ----


    // ---- Billing ----

    #[ink(impl)]
    impl DdcBucket {
        pub fn billing_deposit(&mut self, to: AccountId, cash: Cash) {
            match self.billing_accounts.entry(to) {
                Vacant(e) => {
                    let mut account = BillingAccount::new();
                    account.deposit(cash);
                    e.insert(account);
                }
                Occupied(mut e) => {
                    let account = e.get_mut();
                    account.deposit(cash);
                }
            };
        }

        pub fn billing_withdraw(&mut self, from: AccountId, payable: Payable) -> Result<()> {
            let account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;

            let time_ms = Self::env().block_timestamp();
            account.withdraw(time_ms, payable)?;
            Ok(())
        }

        pub fn billing_get_net(&self, from: AccountId) -> Balance {
            match self.billing_accounts.get(&from) {
                None => 0,
                Some(account) => {
                    let time_ms = Self::env().block_timestamp();
                    account.get_withdrawable(time_ms)
                }
            }
        }

        pub fn billing_balance(&self, account_id: AccountId) -> Balance {
            match self.billing_accounts.get(&account_id) {
                None => 0,
                Some(account) => account.deposit.0,
            }
        }

        pub fn billing_transfer(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
            let (payable, cash) = Self::borrow_payable_cash(amount);
            self.billing_withdraw(from, payable)?;
            self.billing_deposit(to, cash);
            Ok(())
        }

        pub fn billing_start_flow(&mut self, from: AccountId, to: AccountId, rate: Balance) -> Result<FlowId> {
            let start_ms = self.env().block_timestamp();
            let cash_schedule = Schedule::new(start_ms, rate);
            let payable_schedule = cash_schedule.clone();

            let from_account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;
            from_account.lock_schedule(payable_schedule);

            let flow = BillingFlow {
                from,
                to,
                schedule: cash_schedule,
            };
            let flow_id = self.billing_flows.put(flow);
            Ok(flow_id)
        }

        pub fn billing_flow_covered_until(&self, flow_id: FlowId) -> Result<u64> {
            let flow = self.billing_flows.get(flow_id)
                .ok_or(FlowDoesNotExist)?;
            let account = self.billing_accounts.get(&flow.from)
                .ok_or(AccountDoesNotExist)?;

            Ok(account.schedule_covered_until())
        }

        pub fn billing_settle_flow(&mut self, flow_id: FlowId) -> Result<Balance> {
            let now_ms = Self::env().block_timestamp();

            let (flowed_amount, (from, payable), (to, cash)) = {
                let flow = self.billing_flows.get_mut(flow_id)
                    .ok_or(FlowDoesNotExist)?;
                flow.run_until(now_ms)
            };

            let account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;

            account.pay_scheduled(now_ms, payable)?;
            self.billing_deposit(to, cash);

            Ok(flowed_amount)
        }

        pub fn receive_cash() -> Cash {
            Cash(Self::env().transferred_balance())
        }

        pub fn send_cash(destination: AccountId, cash: Cash) -> Result<()> {
            match Self::env().transfer(destination, cash.0) {
                Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
                Ok(_v) => Ok(()),
            }
        }

        pub fn borrow_payable_cash(amount: Balance) -> (Payable, Cash) {
            (Payable(amount), Cash(amount))
        }
    }

    #[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    struct BillingAccount {
        deposit: Cash,
        payable_locked: Balance,
        payable_schedule: Schedule,
    }

    impl BillingAccount {
        pub fn new() -> BillingAccount {
            BillingAccount {
                deposit: Cash(0),
                payable_locked: 0,
                payable_schedule: Schedule::empty(),
            }
        }

        pub fn deposit(&mut self, cash: Cash) {
            self.deposit.0 += cash.0;
        }

        pub fn withdraw(&mut self, time_ms: u64, payable: Payable) -> Result<()> {
            if self.get_withdrawable(time_ms) >= payable.0 {
                self.deposit.0 -= payable.0;
                Ok(())
            } else {
                Err(InsufficientBalance)
            }
        }

        pub fn get_withdrawable(&self, time_ms: u64) -> Balance {
            let consumed = self.payable_locked + self.payable_schedule.value_at_time(time_ms);
            if self.deposit.0 >= consumed {
                self.deposit.0 - consumed
            } else {
                0
            }
        }

        pub fn lock_schedule(&mut self, payable_schedule: Schedule) {
            self.payable_locked += self.payable_schedule.take_value_then_add_rate(payable_schedule);
        }

        pub fn schedule_covered_until(&self) -> u64 {
            self.payable_schedule.time_of_value(self.deposit.0)
        }

        pub fn pay_scheduled(&mut self, now_ms: u64, payable: Payable) -> Result<()> {
            self.unlock_scheduled_amount(now_ms, payable.0);
            self.pay(payable)
        }

        fn pay(&mut self, payable: Payable) -> Result<()> {
            if self.deposit.0 >= payable.0 {
                self.deposit.0 -= payable.0;
                Ok(())
            } else {
                Err(InsufficientBalance)
            }
        }

        fn unlock_scheduled_amount(&mut self, now_ms: u64, unlocked: Balance) {
            self.payable_locked = self.payable_locked
                + self.payable_schedule.take_value_at_time(now_ms)
                - unlocked;
        }
    }

    type FlowId = u32;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    struct BillingFlow {
        from: AccountId,
        to: AccountId,
        schedule: Schedule,
    }

    impl BillingFlow {
        pub fn run_until(&mut self, now_ms: u64) -> (Balance, (AccountId, Payable), (AccountId, Cash)) {
            let flowed_amount = self.schedule.take_value_at_time(now_ms);
            let (payable, cash) = DdcBucket::borrow_payable_cash(flowed_amount);
            (flowed_amount, (self.from, payable), (self.to, cash))
        }
    }

    #[must_use]
    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Schedule {
        rate: Balance,
        start_ms: u64,
    }

    impl Schedule {
        pub fn new(start_ms: u64, rate: Balance) -> Schedule {
            Schedule { rate, start_ms }
        }

        pub fn empty() -> Schedule { Schedule::new(0, 0) }

        pub fn value_at_time(&self, time_ms: u64) -> Balance {
            assert!(time_ms >= self.start_ms);
            let period_ms = (time_ms - self.start_ms) as u128;
            period_ms * self.rate / MS_PER_MONTH
        }

        pub fn time_of_value(&self, value: Balance) -> u64 {
            if self.rate == 0 { return u64::MAX; }
            let duration_ms = value * MS_PER_MONTH / self.rate;
            self.start_ms + duration_ms as u64
        }

        #[must_use]
        pub fn take_value_at_time(&mut self, now_ms: u64) -> Balance {
            let value = self.value_at_time(now_ms);
            self.start_ms = now_ms;
            value
        }

        #[must_use]
        pub fn take_value_then_add_rate(&mut self, to_add: Schedule) -> Balance {
            let accumulated = self.take_value_at_time(to_add.start_ms);
            self.rate += to_add.rate;
            accumulated
        }
    }

    /// Cash represents some value that was taken from someone, and that must be credited to someone.
    #[must_use]
    #[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Cash(pub Balance);

    /// Payable represents some value that was credited to someone, and that must be paid by someone.
    /// Payable must be covered by Cash at all times to guarantee the balance of the contract.
    #[must_use]
    #[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Payable(pub Balance);

    // ---- End Billing ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        ProviderDoesNotExist,
        FlowDoesNotExist,
        AccountDoesNotExist,
        UnauthorizedProvider,
        UnauthorizedOwner,
        TransferFailed,
        InsufficientBalance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl From<Error> for ink_env::Error {
        fn from(_: Error) -> Self {
            ink_env::Error::Unknown
        }
    }

    pub const MS_PER_MONTH: u128 = 31 * 24 * 3600 * 1000;

    // ---- End Utils ----
    #[cfg(test)]
    mod tests;
    #[cfg(test)]
    mod test_utils;
}
