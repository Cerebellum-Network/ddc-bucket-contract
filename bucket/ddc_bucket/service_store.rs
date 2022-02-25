use ink_prelude::{
    vec, vec::Vec,
};
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    collections::Stash,
    collections::Vec as InkVec,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::service::{Service, ServiceId, ServiceParams};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct ServiceStore(pub InkVec<Service>);

impl ServiceStore {
    pub fn create(&mut self, provider_id: AccountId, rent_per_month: Balance, service_params: ServiceParams) -> ServiceId {
        let service_id = self.0.len();
        let service = Service {
            service_id,
            provider_id,
            rent_per_month,
            service_params,
        };
        self.0.push(service);
        service_id
    }

    pub fn get(&self, service_id: ServiceId) -> Result<&Service> {
        self.0.get(service_id).ok_or(ServiceDoesNotExist)
    }

    pub fn list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<Service>, u32) {
        let mut services = Vec::with_capacity(limit as usize);
        for service_id in offset..offset + limit {
            let service = match self.0.get(service_id) {
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
        (services, self.0.len())
    }
}
