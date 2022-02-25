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
        traits::{PackedLayout, SpreadLayout, StorageLayout},
    };
    use scale::{Decode, Encode};

    use account_store::*;
    use billing_account::*;
    use billing_flow::*;
    use bucket::*;
    use bucket_store::*;
    use cash::*;
    use deal::*;
    use deal_store::*;
    use Error::*;
    use flow_store::*;
    use schedule::*;
    use service::*;
    use service_store::*;

    pub mod account_store;
    pub mod billing_account;
    pub mod billing_flow;
    pub mod schedule;
    pub mod cash;
    pub mod service;
    pub mod service_store;
    pub mod bucket;
    pub mod bucket_store;
    pub mod deal;
    pub mod flow_store;
    pub mod deal_store;


    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        deals: DealStore,
        services: ServiceStore,

        billing_accounts: AccountStore,
        billing_flows: FlowStore,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                buckets: BucketStore::default(),
                deals: DealStore::default(),
                services: ServiceStore::default(),
                billing_accounts: AccountStore::default(),
                billing_flows: FlowStore::default(),
            }
        }
    }


    // ---- Bucket ----


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


    impl DdcBucket {
        #[ink(message)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams) -> Result<BucketId> {
            let owner_id = Self::env().caller();
            let bucket_id = self.buckets.create(owner_id, bucket_params);
            Self::env().emit_event(BucketCreated { bucket_id, owner_id });
            Ok(bucket_id)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_add_deal(&mut self, bucket_id: BucketId, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
            // Receive the payable value.
            self.deposit()?;
            let owner_id = Self::env().caller();

            let deal_id = self.deal_create(service_id, deal_params)?;

            let bucket = self.buckets.get_mut(bucket_id)?;
            bucket.only_owner(owner_id)?;
            bucket.deal_ids.push(deal_id);

            Self::env().emit_event(DealCreated { deal_id, bucket_id, service_id });
            Ok(deal_id)
        }

        #[ink(message)]
        pub fn bucket_list_statuses(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            let mut bucket_statuses = Vec::with_capacity(limit as usize);
            for bucket_id in offset..offset + limit {
                let bucket = match self.buckets.0.get(bucket_id) {
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
            (bucket_statuses, self.buckets.0.len())
        }

        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<Bucket> {
            Ok(self.buckets.get(bucket_id)?.clone())
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


    impl DdcBucket {
        pub fn deal_create(&mut self, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
            let payer_id = Self::env().caller();

            // Start the payment flow for a deal.
            let rent_per_month = self.services.get(service_id)?.rent_per_month;
            let flow_id = self.billing_start_flow(payer_id, rent_per_month)?;
            let deal_id = self.deals.create(service_id, flow_id, deal_params);

            Ok(deal_id)
        }

        #[ink(message)]
        pub fn deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
            let deal = self.deals.get(deal_id)?;
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
            let provider_id = self.env().caller();
            let service_id = self.services.create(provider_id, rent_per_month, service_params.clone());
            Self::env().emit_event(ServiceCreated { service_id, provider_id, rent_per_month, service_params });
            Ok(service_id)
        }

        #[ink(message)]
        pub fn service_get(&self, service_id: ServiceId) -> Result<Service> {
            Ok(self.services.get(service_id)?.clone())
        }

        #[ink(message)]
        pub fn service_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<Service>, u32) {
            self.services.list(offset, limit, filter_provider_id)
        }

        #[ink(message)]
        pub fn provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
            let caller = self.env().caller();

            let (flow_id, service_id) = {
                let deal = self.deals.get(deal_id)?;
                (deal.flow_id, deal.service_id)
            };

            // Find where to distribute the revenues.
            let revenue_account_id = {
                let service = self.services.get(service_id)?;
                // Authorize only the service owner to trigger the distribution.
                service.only_owner(caller)?;
                service.revenue_account_id()
            };

            let cash = self.billing_settle_flow(flow_id)?;

            Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, deal_id, value: cash.peek() });

            Self::send_cash(revenue_account_id, cash)
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

            self.billing_accounts.deposit(account_id, cash);
            Self::env().emit_event(Deposit { account_id, value });
            Ok(())
        }

        pub fn billing_withdraw(&mut self, from: AccountId, payable: Payable) -> Result<()> {
            let account = self.billing_accounts.0.get_mut(&from)
                .ok_or(InsufficientBalance)?;

            let time_ms = Self::env().block_timestamp();
            account.withdraw(time_ms, payable)?;
            Ok(())
        }

        pub fn billing_get_net(&self, from: AccountId) -> Balance {
            match self.billing_accounts.0.get(&from) {
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
            self.billing_accounts.deposit(to, cash);
            Ok(())
        }

        pub fn billing_start_flow(&mut self, from: AccountId, rate: Balance) -> Result<FlowId> {
            let start_ms = self.env().block_timestamp();
            let cash_schedule = Schedule::new(start_ms, rate);
            let payable_schedule = cash_schedule.clone();

            let from_account = self.billing_accounts.get_mut(&from)?;
            from_account.lock_schedule(payable_schedule);

            let flow_id = self.billing_flows.create(from, cash_schedule);
            Ok(flow_id)
        }

        pub fn billing_flow_covered_until(&self, flow_id: FlowId) -> Result<u64> {
            let flow = self.billing_flows.get(flow_id)?;
            let account = self.billing_accounts.get(&flow.from)?;
            Ok(account.schedule_covered_until())
        }

        pub fn billing_settle_flow(&mut self, flow_id: FlowId) -> Result<Cash> {
            let now_ms = Self::env().block_timestamp();

            let flow = self.billing_flows.get_mut(flow_id)?;
            let flowed_amount = flow.schedule.take_value_at_time(now_ms);
            let (payable, cash) = Cash::borrow_payable_cash(flowed_amount);

            let account = self.billing_accounts.get_mut(&flow.from)?;
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
