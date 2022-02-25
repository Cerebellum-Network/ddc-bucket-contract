#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_bucket {
    use ink_prelude::{vec, vec::Vec};
    use scale::{Decode, Encode};

    use account::store::*;
    use bucket::{entity::*, store::*};
    use cash::*;
    use deal::{entity::*, store::*};
    use Error::*;
    use flow::{entity::*, store::*};
    use schedule::*;
    use service::{entity::*, store::*};

    pub mod billing;
    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod service;
    pub mod bucket;
    pub mod deal;


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
            self.message_bucket_create(bucket_params)
        }

        #[ink(message, payable)]
        pub fn bucket_add_deal(&mut self, bucket_id: BucketId, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
            self.message_bucket_add_deal(bucket_id, service_id, deal_params)
        }

        #[ink(message)]
        pub fn bucket_list_statuses(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list_statuses(offset, limit, filter_owner_id)
        }

        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<Bucket> {
            Ok(self.buckets.get(bucket_id)?.clone())
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            self.message_bucket_get_status(bucket_id)
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
            self.message_service_create(rent_per_month, service_params)
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
        #[ink(message, payable)]
        pub fn deposit(&mut self) -> Result<()> {
            self.message_deposit()
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
