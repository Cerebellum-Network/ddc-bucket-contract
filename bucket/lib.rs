#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file

use ink_lang as ink;

#[ink::contract]
pub mod ddc_bucket {
    use core::cmp::min;

    use ink_prelude::{
        string::String,
        vec, vec::Vec,
    };
    use ink_storage::{
        collections::HashMap,
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct DdcBucket {
        providers: HashMap<AccountId, Provider>,
        buckets: Stash<Bucket>,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { providers: HashMap::new(), buckets: Stash::new() }
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
        deposit: Balance,

        provider_id: AccountId,
        rent_per_month: Balance,
        rent_start_ms: u64,
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
            let value = self.env().transferred_balance();

            let now_ms = Self::env().block_timestamp();
            let bucket = Bucket {
                owner_id: self.env().caller(),
                deposit: value,

                provider_id,
                rent_per_month: self.get_provider_rent(provider_id)?,
                rent_start_ms: now_ms,
            };
            let bucket_id = self.buckets.put(bucket);

            Self::env().emit_event(BucketCreated { bucket_id });
            Self::env().emit_event(BucketTopup { bucket_id, value });
            Ok(bucket_id)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn bucket_topup(&mut self, bucket_id: BucketId) -> Result<()> {
            let value = self.env().transferred_balance();

            match self.buckets.get_mut(bucket_id) {
                None => Err(Error::BucketDoesNotExist),
                Some(bucket) => {
                    bucket.deposit += value;
                    Self::env().emit_event(BucketTopup { bucket_id, value });
                    Ok(())
                }
            }
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            let bucket = self.buckets.get(bucket_id)
                .ok_or(Error::BucketDoesNotExist)?;

            let status = BucketStatus {
                provider_id: bucket.provider_id,
                estimated_rent_end_ms: Self::estimate_rent_end_ms(bucket),
                writer_ids: vec![bucket.owner_id],
            };

            Ok(status)
        }

        fn estimate_rent_end_ms(bucket: &Bucket) -> u64 {
            let paid_duration_ms = bucket.deposit * MS_PER_MONTH / bucket.rent_per_month;
            let end_ms = bucket.rent_start_ms + paid_duration_ms as u64;
            end_ms
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

            let bucket = self.buckets.get_mut(bucket_id)
                .ok_or(Error::BucketDoesNotExist)?;

            if bucket.provider_id != provider_id {
                return Err(Error::UnauthorizedProvider);
            }

            let now_ms = Self::env().block_timestamp();
            let period_ms = (now_ms - bucket.rent_start_ms) as u128;
            let earned = bucket.rent_per_month * period_ms / MS_PER_MONTH;
            let to_withdraw = min(earned, bucket.deposit);

            bucket.rent_start_ms = now_ms;
            bucket.deposit -= to_withdraw;

            Self::transfer(provider_id, to_withdraw)?;

            Self::env().emit_event(ProviderWithdraw { provider_id, bucket_id, value: to_withdraw });
            Ok(())
        }
    }
    // ---- End Provider ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        ProviderDoesNotExist,
        UnauthorizedProvider,
        TransferFailed,
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
