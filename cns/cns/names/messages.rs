//! The operations on names and records.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::string::String;

use crate::cns::{AllocateName, CNS, Result, SetPayload};

use super::entity::Record;

impl CNS {
    pub fn message_claim_name(&mut self, name: String) -> Result<()> {
        let owner_id = Self::env().caller();

        let record = Record::new(owner_id);
        self.name_store.create(name.clone(), record)?;

        Self::env().emit_event(AllocateName { name, owner_id });
        Ok(())
    }

    pub fn message_set_payload(&mut self, name: String, payload: String) -> Result<()> {
        let caller_id = Self::env().caller();

        let record = self.name_store.get_mut(&name)?;
        record.only_owner(caller_id)?;
        record.set_payload(payload.clone())?;

        Self::env().emit_event(SetPayload { name, payload });
        Ok(())
    }
}
