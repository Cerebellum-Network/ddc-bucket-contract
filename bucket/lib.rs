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
        repbucks: Stash<RepBuck>,
        buckets: Stash<Bucket>,
        services: HashMap<ServiceId, Service>,

        billing_accounts: HashMap<AccountId, BillingAccount>,
        billing_flows: Stash<BillingFlow>,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                repbucks: Stash::new(),
                buckets: Stash::new(),
                services: HashMap::new(),
                billing_accounts: HashMap::new(),
                billing_flows: Stash::new(),
            }
        }
    }


    // ---- RepBucket ----
    pub type RepBuckId = u32;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct RepBuck {
        owner_id: AccountId,
        bucket_ids: Vec<BucketId>,
    }

    impl RepBuck {
        pub fn only_owner(&self, caller: AccountId) -> Result<()> {
            if self.owner_id == caller { Ok(()) } else { Err(UnauthorizedOwner) }
        }
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn repbuck_create(&mut self) -> Result<RepBuckId> {
            let caller = Self::env().caller();

            let repbuck = RepBuck {
                owner_id: caller,
                bucket_ids: Vec::new(),
            };
            let repbuck_id = self.repbucks.put(repbuck);
            Ok(repbuck_id)
        }

        #[ink(message)]
        pub fn repbuck_attach_service(&mut self, repbuck_id: RepBuckId, service_id: ServiceId) -> Result<BucketId> {
            // bucket_create captures the sent value.
            let bucket_id = self.bucket_create(service_id)?;

            let repbuck = self.repbucks.get_mut(repbuck_id)
                .ok_or(RepbuckDoesNotExist)?;
            repbuck.only_owner(Self::env().caller())?;

            repbuck.bucket_ids.push(bucket_id);
            Ok(bucket_id)
        }

        #[ink(message)]
        pub fn repbuck_get(&self, repbuck_id: RepBuckId) -> Result<RepBuck> {
            self.repbucks.get(repbuck_id)
                .cloned().ok_or(RepbuckDoesNotExist)
        }
    }


    // ---- End RepBucket ----


    // ---- Bucket ----
    pub type BucketId = u32;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Bucket {
        owner_id: AccountId,
        service_id: ServiceId,
        flow_id: FlowId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketCreated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        service_id: ServiceId,
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
        service_id: ServiceId,
        estimated_rent_end_ms: u64,
        writer_ids: Vec<AccountId>,
    }

    impl DdcBucket {
        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_create(&mut self, service_id: ServiceId) -> Result<BucketId> {
            // Receive the payable value.
            self.deposit()?;
            let caller = Self::env().caller();

            // Start the payment flow for a bucket.
            let service = self.service_get_info(service_id)?;
            let flow_id = self.billing_start_flow(caller, service.rent_per_month)?;

            // Create a new bucket.
            let bucket = Bucket {
                owner_id: caller,
                service_id,
                flow_id,
            };
            let bucket_id = self.buckets.put(bucket);

            Self::env().emit_event(BucketCreated { bucket_id, service_id });
            Ok(bucket_id)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_topup(&mut self, _bucket_id: BucketId) -> Result<()> {
            self.deposit()
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self) -> Result<()> {
            // Receive the payable value.
            let cash = Self::receive_cash();
            let value = cash.peek();
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
                service_id: bucket.service_id,
                estimated_rent_end_ms,
                writer_ids: vec![bucket.owner_id],
            })
        }
    }
    // ---- End Bucket ----


    // ---- Provider ----
    pub type ProviderId = AccountId;
    pub type ServiceId = (AccountId, u32);

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Service {
        provider_id: ProviderId,
        rent_per_month: Balance,
        description: String,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ServiceSetInfo {
        #[ink(topic)]
        provider_id: AccountId,
        // TODO: remove?
        #[ink(topic)]
        service_id: ServiceId,
        rent_per_month: Balance,
        description: String,
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
        pub fn service_set_info(&mut self, service_id: ServiceId, rent_per_month: Balance, description: String) -> Result<()> {
            let provider_id = self.env().caller();
            Service::only_owner(service_id, provider_id)?;

            self.services.insert(service_id, Service {
                provider_id,
                rent_per_month,
                description: description.clone(),
            });

            Self::env().emit_event(ServiceSetInfo { provider_id, service_id, rent_per_month, description });
            Ok(())
        }

        #[ink(message)]
        pub fn service_get_info(&self, service_id: ServiceId) -> Result<Service> {
            self.services.get(&service_id)
                .cloned()
                .ok_or(ServiceDoesNotExist)
        }

        #[ink(message)]
        pub fn provider_withdraw(&mut self, bucket_id: BucketId) -> Result<()> {
            let caller = self.env().caller();

            let (flow_id, service_id) = {
                let bucket = self.buckets.get(bucket_id)
                    .ok_or(BucketDoesNotExist)?;
                (bucket.flow_id, bucket.service_id)
            };

            // Find where to distribute the revenues.
            let revenue_account_id = {
                let service = self.service_get_info(service_id)?;
                // Authorize only the service owner to trigger the distribution.
                Service::only_owner(service_id, caller)?;
                service.revenue_account_id()
            };

            let cash = self.billing_settle_flow(flow_id)?;

            Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, bucket_id, value: cash.peek() });

            Self::send_cash(revenue_account_id, cash)
        }
    }

    impl Service {
        pub fn revenue_account_id(&self) -> AccountId {
            self.provider_id
        }

        pub fn only_owner(service_id: ServiceId, provider_id: AccountId) -> Result<()> {
            if service_id.0 == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
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
                Some(account) => account.deposit.peek(),
            }
        }

        pub fn billing_transfer(&mut self, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
            let (payable, cash) = Cash::borrow_payable_cash(amount);
            self.billing_withdraw(from, payable)?;
            self.billing_deposit(to, cash);
            Ok(())
        }

        pub fn billing_start_flow(&mut self, from: AccountId, rate: Balance) -> Result<FlowId> {
            let start_ms = self.env().block_timestamp();
            let cash_schedule = Schedule::new(start_ms, rate);
            let payable_schedule = cash_schedule.clone();

            let from_account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;
            from_account.lock_schedule(payable_schedule);

            let flow = BillingFlow {
                from,
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

        pub fn billing_settle_flow(&mut self, flow_id: FlowId) -> Result<Cash> {
            let now_ms = Self::env().block_timestamp();

            let flow = self.billing_flows.get_mut(flow_id)
                .ok_or(FlowDoesNotExist)?;
            let flowed_amount = flow.schedule.take_value_at_time(now_ms);
            let (payable, cash) = Cash::borrow_payable_cash(flowed_amount);

            let account = self.billing_accounts.get_mut(&flow.from)
                .ok_or(InsufficientBalance)?;

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
            self.deposit.increase(cash);
        }

        pub fn withdraw(&mut self, time_ms: u64, payable: Payable) -> Result<()> {
            if self.get_withdrawable(time_ms) >= payable.peek() {
                self.deposit.pay_unchecked(payable);
                Ok(())
            } else {
                Err(InsufficientBalance)
            }
        }

        pub fn get_withdrawable(&self, time_ms: u64) -> Balance {
            let deposit = self.deposit.peek();
            let consumed = self.payable_locked + self.payable_schedule.value_at_time(time_ms);
            if deposit >= consumed {
                deposit - consumed
            } else {
                0
            }
        }

        pub fn lock_schedule(&mut self, payable_schedule: Schedule) {
            self.payable_locked += self.payable_schedule.take_value_then_add_rate(payable_schedule);
        }

        pub fn schedule_covered_until(&self) -> u64 {
            self.payable_schedule.time_of_value(self.deposit.peek())
        }

        pub fn pay_scheduled(&mut self, now_ms: u64, payable: Payable) -> Result<()> {
            self.unlock_scheduled_amount(now_ms, payable.peek());
            self.pay(payable)
        }

        fn pay(&mut self, payable: Payable) -> Result<()> {
            if self.deposit.peek() >= payable.peek() {
                self.deposit.pay_unchecked(payable);
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
        schedule: Schedule,
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

    impl Cash {
        pub fn borrow_payable_cash(amount: Balance) -> (Payable, Cash) {
            (Payable(amount), Cash(amount))
        }

        #[must_use]
        pub fn consume(self) -> Balance { self.0 }

        pub fn peek(&self) -> Balance { self.0 }

        pub fn increase(&mut self, cash: Cash) {
            self.0 += cash.consume();
        }

        pub fn pay_unchecked(&mut self, payable: Payable) {
            self.0 -= payable.consume();
        }
    }

    impl Payable {
        #[must_use]
        pub fn consume(self) -> Balance { self.0 }

        pub fn peek(&self) -> Balance { self.0 }
    }

    // ---- End Billing ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        RepbuckDoesNotExist,
        BucketDoesNotExist,
        ServiceDoesNotExist,
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
