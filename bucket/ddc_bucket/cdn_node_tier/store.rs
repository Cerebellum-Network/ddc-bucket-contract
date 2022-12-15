use ink_storage::{collections::{HashMap}, traits};

use crate::ddc_bucket::{Error::*, Result};
use crate::ddc_bucket::cdn_node_tier::entity::CdnNodeTier;
use crate::ddc_bucket::{NodeId};

#[derive(traits::SpreadLayout, Default, Debug)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout))]
pub struct CdnNodeTierStore(pub HashMap<NodeId, CdnNodeTier>);

impl CdnNodeTierStore {
    pub fn toggle_tier(&mut self, cdn_node_id: NodeId) -> Result<()> {
        if self.0.contains_key(&cdn_node_id) {
            let cdn_node_tier = self.0.get_mut(&cdn_node_id).ok_or(ParamsDoesNotExist)?;
            let mut new_tier = 1;
            if cdn_node_tier.tier_id == 1 {
                new_tier = 2
            }
            cdn_node_tier.tier_id = new_tier;
            return Ok(());
        }

        self.0.insert(cdn_node_id, CdnNodeTier { cdn_node_id, tier_id: 2 });

        Ok(())
    }

    pub fn get(&self, cdn_node_id: NodeId) -> Result<&CdnNodeTier> {
        self.0.get(&cdn_node_id).ok_or(ParamsDoesNotExist)
    }
}
