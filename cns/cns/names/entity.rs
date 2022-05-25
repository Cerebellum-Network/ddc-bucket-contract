//! Record is the data structure attached to a name.

use ink_prelude::string::String;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::cns::{AccountId, Error::*, Result};

pub const PAYLOAD_MAX_LEN: usize = 10_000;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Record {
    pub owner_id: AccountId,
    pub payload: String,
}

impl Record {
    pub fn new(owner_id: AccountId) -> Self {
        Self { owner_id, payload: "".to_string() }
    }

    pub fn set_payload(&mut self, payload: String) -> Result<()> {
        if payload.len() > PAYLOAD_MAX_LEN {
            return Err(PayloadTooLong);
        }
        self.payload = payload;
        Ok(())
    }

    pub fn only_owner(&self, caller_id: AccountId) -> Result<()> {
        if caller_id == self.owner_id {
            Ok(())
        } else {
            Err(Unauthorized)
        }
    }
}