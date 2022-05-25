use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::string::String;

use crate::cns::{AllocateName, CNS, Result};

impl CNS {
    pub fn message_claim_name(&mut self, name: String) -> Result<()> {
        let owner_id = Self::env().caller();
        Self::env().emit_event(AllocateName { name, owner_id });
        Ok(())
    }
}
