#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file

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

        fn transfer(destination: AccountId, amount: Balance) -> Result<()> {
            match Self::env().transfer(destination, amount) {
                Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
                Ok(_v) => Ok(()),
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
    pub struct BucketTopup {
        #[ink(topic)]
        bucket_id: BucketId,
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
            let caller = self.env().caller();

            let rent_per_month = self.get_provider_rent(provider_id)?;
            let flow_id = self.billing_start_flow(caller, provider_id, rent_per_month);

            let bucket = Bucket {
                owner_id: self.env().caller(),
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
        pub fn bucket_topup(&mut self, bucket_id: BucketId) -> Result<()> {
            let caller = self.env().caller();
            let value = self.env().transferred_balance();
            self.billing_fund(caller, value);

            match self.buckets.get_mut(bucket_id) {
                None => Err(Error::BucketDoesNotExist),
                Some(bucket) => {
                    if caller != bucket.owner_id { return Err(UnauthorizedOwner); }
                    Self::env().emit_event(BucketTopup { bucket_id, value });
                    Ok(())
                }
            }
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            let bucket = self.buckets.get(bucket_id)
                .ok_or(Error::BucketDoesNotExist)?;

            let estimated_rent_end_ms = self.billing_get_flow_end(bucket.flow_id)?;

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

            let value_flowed = self.billing_settle_flow(flow_id)?;
            self.billing_take(provider_id, value_flowed)?;
            Self::transfer(provider_id, value_flowed)?;

            Self::env().emit_event(ProviderWithdraw { provider_id, bucket_id, value: value_flowed });
            Ok(())
        }
    }
    // ---- End Provider ----


    // ---- Billing ----
    type FlowId = u32;

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    struct BillingAccount {
        balance: Balance,
    }

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    struct BillingFlow {
        from: AccountId,
        to: AccountId,
        accu: Accumulator,
    }

    #[ink(impl)]
    impl DdcBucket {
        pub fn billing_fund(&mut self, to: AccountId, received_value: Balance) {
            match self.billing_accounts.entry(to) {
                Vacant(e) => {
                    e.insert(BillingAccount {
                        balance: received_value,
                    });
                }
                Occupied(mut e) => {
                    let account = e.get_mut();
                    account.balance += received_value;
                }
            };
        }

        pub fn billing_take(&mut self, from: AccountId, value: Balance) -> Result<()> {
            let account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;
            let balance = account.balance;
            if balance < value { return Err(InsufficientBalance); }
            account.balance = balance - value;
            Ok(())
        }

        pub fn billing_take_all(&mut self, from: AccountId) -> Result<Balance> {
            let account = self.billing_accounts.get_mut(&from)
                .ok_or(InsufficientBalance)?;
            let value_to_send = account.balance;
            account.balance = 0;
            Ok(value_to_send)
        }

        pub fn billing_balance(&self, account_id: AccountId) -> Balance {
            match self.billing_accounts.get(&account_id) {
                None => 0,
                Some(account) => account.balance,
            }
        }

        pub fn billing_transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.billing_take(from, value)?;
            self.billing_fund(to, value);
            Ok(())
        }

        pub fn billing_start_flow(&mut self, from: AccountId, to: AccountId, rate: Balance) -> FlowId {
            let start_ms = self.env().block_timestamp();
            let flow = BillingFlow {
                from,
                to,
                accu: Accumulator { rate, start_ms },
            };
            let flow_id = self.billing_flows.put(flow);
            flow_id
        }

        pub fn billing_get_flow_end(&self, flow_id: FlowId) -> Result<u64> {
            let flow = self.billing_flows.get(flow_id)
                .ok_or(FlowDoesNotExist)?;
            let flow_deposit = self.billing_balance(flow.from);
            let end_ms = flow.accu.time_of_value(flow_deposit);
            Ok(end_ms)
        }

        pub fn billing_settle_flow(&mut self, flow_id: FlowId) -> Result<Balance> {
            let (from, to, value_flowed) = {
                let flow = self.billing_flows.get_mut(flow_id)
                    .ok_or(FlowDoesNotExist)?;

                let now_ms = Self::env().block_timestamp();
                let value_flowed = flow.accu.take_value_at_time(now_ms);

                (flow.from, flow.to, value_flowed)
            };

            self.billing_transfer(from, to, value_flowed)?;
            Ok(value_flowed)
        }
    }

    #[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Accumulator {
        rate: Balance,
        start_ms: u64,
    }

    impl Accumulator {
        pub fn value_at_time(&self, time_ms: u64) -> Balance {
            assert!(time_ms >= self.start_ms);
            let period_ms = (time_ms - self.start_ms) as u128;
            period_ms * self.rate / MS_PER_MONTH
        }

        pub fn time_of_value(&self, value: Balance) -> u64 {
            let duration_ms = value * MS_PER_MONTH / self.rate;
            self.start_ms + duration_ms as u64
        }

        pub fn take_value_at_time(&mut self, now_ms: u64) -> Balance {
            let value = self.value_at_time(now_ms);
            self.start_ms = now_ms;
            value
        }
    }

    // ---- End Billing ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        ProviderDoesNotExist,
        FlowDoesNotExist,
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
