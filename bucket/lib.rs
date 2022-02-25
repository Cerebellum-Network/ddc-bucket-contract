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
        collections::Vec as InkVec,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};
    //use ink_lang::{Env, StaticEnv, EnvAccess, ContractEnv};

    use Error::*;

    #[ink(storage)]
    pub struct DdcBucket {
        buckets: InkVec<Bucket>,
        deals: Stash<Deal>,
        services: InkVec<Service>,

        billing_accounts: HashMap<AccountId, BillingAccount>,
        billing_flows: Stash<BillingFlow>,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                buckets: InkVec::new(),
                deals: Stash::new(),
                services: InkVec::new(),
                billing_accounts: HashMap::new(),
                billing_flows: Stash::new(),
            }
        }
    }


    // ---- Bucket ----
    pub type BucketId = u32;
    pub type BucketParams = String;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Bucket {
        owner_id: AccountId,
        deal_ids: Vec<DealId>,
        bucket_params: BucketParams,
    }

    #[derive(Clone, PartialEq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct BucketStatus {
        bucket_id: BucketId,
        bucket: Bucket,
        writer_ids: Vec<AccountId>,
        deal_statuses: Vec<DealStatus>,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketCreated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        owner_id: AccountId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct DealCreated {
        #[ink(topic)]
        deal_id: DealId,
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        service_id: ServiceId,
    }

    impl Bucket {
        pub fn only_owner(&self, caller: AccountId) -> Result<()> {
            if self.owner_id == caller { Ok(()) } else { Err(UnauthorizedOwner) }
        }
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams) -> Result<BucketId> {
            let owner_id = Self::env().caller();

            let bucket = Bucket {
                owner_id,
                deal_ids: Vec::new(),
                bucket_params,
            };
            let bucket_id = self.buckets.len();
            self.buckets.push(bucket);
            Self::env().emit_event(BucketCreated { bucket_id, owner_id });
            Ok(bucket_id)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_add_deal(&mut self, bucket_id: BucketId, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
            // Receive the payable value.
            self.deposit()?;

            let deal_id = self.deal_create(service_id, deal_params)?;

            let bucket = self.buckets.get_mut(bucket_id)
                .ok_or(BucketDoesNotExist)?;
            bucket.only_owner(Self::env().caller())?;

            bucket.deal_ids.push(deal_id);
            Self::env().emit_event(DealCreated { deal_id, bucket_id, service_id });
            Ok(deal_id)
        }

        #[ink(message)]
        pub fn bucket_list_statuses(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            let mut bucket_statuses = Vec::with_capacity(limit as usize);
            for bucket_id in offset..offset + limit {
                let bucket = match self.buckets.get(bucket_id) {
                    None => break, // No more buckets, stop.
                    Some(bucket) => bucket,
                };
                // Apply the filter if given.
                if let Some(owner_id) = filter_owner_id {
                    if owner_id != bucket.owner_id {
                        continue; // Skip non-matches.
                    }
                }
                // Collect all the details of the bucket.
                match self.bucket_collect_status(bucket_id, bucket.clone()) {
                    Err(_) => continue, // Skip on unexpected error.
                    Ok(status) =>
                        bucket_statuses.push(status),
                };
            }
            (bucket_statuses, self.buckets.len())
        }

        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<Bucket> {
            self.buckets.get(bucket_id)
                .cloned().ok_or(BucketDoesNotExist)
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            let bucket = self.bucket_get(bucket_id)?;
            self.bucket_collect_status(bucket_id, bucket)
        }

        fn bucket_collect_status(&self, bucket_id: BucketId, bucket: Bucket) -> Result<BucketStatus> {
            let writer_ids = vec![bucket.owner_id];

            let mut deal_statuses = Vec::with_capacity(bucket.deal_ids.len());
            for deal_id in bucket.deal_ids.iter() {
                deal_statuses.push(self.deal_get_status(*deal_id)?);
            }

            Ok(BucketStatus { bucket_id, bucket, writer_ids, deal_statuses })
        }
    }
    // ---- End Bucket ----


    // ---- Deal ----
    pub type DealId = u32;
    pub type DealParams = String;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Deal {
        service_id: ServiceId,
        flow_id: FlowId,
        deal_params: DealParams,
    }

    #[derive(Clone, PartialEq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct DealStatus {
        service_id: ServiceId,
        estimated_rent_end_ms: u64,
        deal_params: DealParams,
    }

    impl DdcBucket {
        pub fn deal_create(&mut self, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
            let payer_id = Self::env().caller();

            // Start the payment flow for a deal.
            let service = self.service_get(service_id)?;
            let flow_id = self.billing_start_flow(payer_id, service.rent_per_month)?;

            // Create a new deal.
            let deal = Deal {
                service_id,
                flow_id,
                deal_params,
            };
            let deal_id = self.deals.put(deal);
            Ok(deal_id)
        }

        #[ink(message)]
        pub fn deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
            let deal = self.deals.get(deal_id)
                .ok_or(Error::DealDoesNotExist)?;

            let estimated_rent_end_ms = self.billing_flow_covered_until(deal.flow_id)?;

            Ok(DealStatus {
                service_id: deal.service_id,
                estimated_rent_end_ms,
                deal_params: deal.deal_params.clone(),
            })
        }
    }
    // ---- End Deal ----


    // ---- Provider ----
    pub type ProviderId = AccountId;
    pub type ServiceId = u32;
    pub type ServiceParams = String;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Service {
        service_id: ServiceId,
        provider_id: ProviderId,
        rent_per_month: Balance,
        service_params: ServiceParams,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ServiceCreated {
        #[ink(topic)]
        service_id: ServiceId,
        #[ink(topic)]
        provider_id: AccountId,
        rent_per_month: Balance,
        service_params: ServiceParams,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ProviderWithdraw {
        #[ink(topic)]
        provider_id: AccountId,
        #[ink(topic)]
        deal_id: DealId,
        value: Balance,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn service_create(&mut self, rent_per_month: Balance, service_params: ServiceParams) -> Result<ServiceId> {
            let service_id = self.services.len();
            let provider_id = self.env().caller();
            let service = Service {
                service_id,
                provider_id,
                rent_per_month,
                service_params: service_params.clone(),
            };

            self.services.push(service);
            Self::env().emit_event(ServiceCreated { service_id, provider_id, rent_per_month, service_params });
            Ok(service_id)
        }

        #[ink(message)]
        pub fn service_get(&self, service_id: ServiceId) -> Result<Service> {
            self.services.get(service_id)
                .cloned()
                .ok_or(ServiceDoesNotExist)
        }

        #[ink(message)]
        pub fn service_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<Service>, u32) {
            let mut services = Vec::with_capacity(limit as usize);
            for service_id in offset..offset + limit {
                let service = match self.services.get(service_id) {
                    None => break, // No more services, stop.
                    Some(service) => service,
                };
                // Apply the filter if given.
                if let Some(provider_id) = filter_provider_id {
                    if provider_id != service.provider_id {
                        continue; // Skip non-matches.
                    }
                }
                services.push(service.clone());
            }
            (services, self.services.len())
        }

        #[ink(message)]
        pub fn provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
            let caller = self.env().caller();

            let (flow_id, service_id) = {
                let deal = self.deals.get(deal_id)
                    .ok_or(DealDoesNotExist)?;
                (deal.flow_id, deal.service_id)
            };

            // Find where to distribute the revenues.
            let revenue_account_id = {
                let service = self.service_get(service_id)?;
                // Authorize only the service owner to trigger the distribution.
                service.only_owner(caller)?;
                service.revenue_account_id()
            };

            let cash = self.billing_settle_flow(flow_id)?;

            Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, deal_id, value: cash.peek() });

            Self::send_cash(revenue_account_id, cash)
        }
    }

    impl Service {
        pub fn revenue_account_id(&self) -> AccountId {
            self.provider_id
        }

        pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
            if self.provider_id == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
        }
    }
    // ---- End Provider ----


    // ---- Billing ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Deposit {
        #[ink(topic)]
        account_id: AccountId,
        value: Balance,
    }

    impl DdcBucket {
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

    pub mod schedule;
    use schedule::*;

    pub mod cash;
    use cash::*;

    // ---- End Billing ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        DealDoesNotExist,
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


    // ---- End Utils ----
    #[cfg(test)]
    mod tests;
    #[cfg(test)]
    mod test_utils;
}
