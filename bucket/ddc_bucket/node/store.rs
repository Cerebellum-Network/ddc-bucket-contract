//! The store where to create and access Nodes.

use ink_storage::{collections::Vec as InkVec, traits};

use crate::ddc_bucket::cdn_cluster::entity::ClusterId;
use crate::ddc_bucket::cluster::entity::Cluster;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use ink_storage::collections::HashMap;

use super::entity::{Node, NodeId, NodeTag};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NodeStore {
    pub account_node: HashMap<AccountId, NodeId>,
    pub nodes: InkVec<Node>,
}

impl NodeStore {
    pub fn create(
        &mut self,
        provider_id: AccountId,
        rent_per_month: Balance,
        capacity: Resource,
        node_tag: NodeTag,
        pubkey: AccountId,
    ) -> Result<NodeId> {
        let node_id = self.nodes.len();

        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            node_tag,
            cluster_id: ClusterId::default(),
        };

        let exists = self.account_node.contains_key(&pubkey);
        if exists {
            return Err(NodeAlreadyExists);
        }

        self.nodes.push(node);
        self.account_node.insert(pubkey, node_id);

        Ok(node_id)
    }

    pub fn get_by_pub_key(&self, pubkey: AccountId) -> Result<&NodeId> {
        self.account_node.get(&pubkey).ok_or(NodeDoesNotExist)
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(node_id).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes.get_mut(node_id).ok_or(NodeDoesNotExist)
    }

    pub fn assign_cluster_id(&mut self, node_id: NodeId, cluster_id: ClusterId) {
        self.nodes
            .get_mut(node_id)
            .unwrap()
            .assign_cluster_id(cluster_id)
    }
}
