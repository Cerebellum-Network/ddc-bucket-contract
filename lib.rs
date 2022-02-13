#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file

use ink_lang as ink;

#[ink::contract]
mod ddc_bucket_contract {
    use core::cmp::min;

    use ink_prelude::vec;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::HashMap,
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct DdcBucketContract {
        providers: HashMap<AccountId, Provider>,
        buckets: Stash<Bucket>,
    }

    #[derive(Copy, Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Provider {
        rent_per_month: Balance,
    }

    pub type BucketId = u32;

    #[derive(Copy, Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Bucket {
        owner_id: AccountId,
        deposit: Balance,

        provider_id: AccountId,
        rent_per_month: Balance,
        rent_start_ms: u64,
    }

    #[derive(Clone, PartialEq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct BucketStatus {
        provider_id: AccountId,
        estimated_rent_end_ms: u64,
        writer_ids: Vec<AccountId>,
    }

    impl DdcBucketContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { providers: HashMap::new(), buckets: Stash::new() }
        }


        // ---- As Consumer ----
        #[ink(message)]
        pub fn create_bucket(&mut self, provider_id: AccountId) -> Result<BucketId> {
            let now_ms = Self::env().block_timestamp();
            let bucket = Bucket {
                owner_id: self.env().caller(),
                deposit: self.env().transferred_balance(),

                provider_id,
                rent_per_month: self.get_provider_rent(provider_id)?,
                rent_start_ms: now_ms,
            };
            let bucket_id = self.buckets.put(bucket);
            Ok(bucket_id)
        }

        #[ink(message)]
        pub fn topup_bucket(&mut self, bucket_id: BucketId) -> Result<()> {
            let value = self.env().transferred_balance();

            match self.buckets.get_mut(bucket_id) {
                None => Err(Error::BucketDoesNotExist),
                Some(bucket) => {
                    bucket.deposit += value;
                    Ok(())
                }
            }
        }

        #[ink(message)]
        pub fn get_bucket_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            let bucket = self.buckets.get(bucket_id)
                .ok_or(Error::BucketDoesNotExist)?;

            let status = BucketStatus {
                provider_id: bucket.provider_id,
                estimated_rent_end_ms: Self::estimate_rent_end_ms(bucket),
                writer_ids: vec![bucket.owner_id],
            };

            Ok(status)
        }

        pub fn estimate_rent_end_ms(bucket: &Bucket) -> u64 {
            let paid_duration_ms = bucket.deposit * MS_PER_MONTH / bucket.rent_per_month;
            let end_ms = bucket.rent_start_ms + paid_duration_ms as u64;
            end_ms
        }

        pub fn get_provider_rent(&self, provider_id: AccountId) -> Result<Balance> {
            let provider = self.providers.get(&provider_id)
                .ok_or(Error::ProviderDoesNotExist)?;
            Ok(provider.rent_per_month)
        }


        // ---- As Provider ----

        #[ink(message)]
        pub fn provider_set_info(&mut self, rent_per_month: Balance) -> Result<()> {
            let provider_id = self.env().caller();
            self.providers.insert(provider_id, Provider {
                rent_per_month,
            });
            Ok(())
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

            Ok(())
        }

        fn transfer(destination: AccountId, amount: Balance) -> Result<()> {
            match Self::env().transfer(destination, amount) {
                Err(_e) => Err(Error::TransferFailed),
                Ok(_v) => Ok(()),
            }
        }
    }

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
}

#[cfg(test)]
mod tests;
