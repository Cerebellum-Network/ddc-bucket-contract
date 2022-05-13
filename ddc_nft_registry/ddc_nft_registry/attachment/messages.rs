use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::string::String;

use crate::ddc_nft_registry::{Attach, DdcNftRegistry, Result};

impl DdcNftRegistry {
    pub fn message_attach(&mut self, nft_id: String, asset_id: String, proof: String) -> Result<()> {
        let reporter_id = Self::env().caller();
        Self::env().emit_event(Attach { reporter_id, nft_id, asset_id, proof });
        Ok(())
    }
}
