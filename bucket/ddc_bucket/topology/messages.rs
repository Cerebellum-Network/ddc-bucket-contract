//! The public interface to manage Nodes.

use crate::ddc_bucket::{NodeKey, ClusterId, VNodeToken, DdcBucket, Result};
use ink_prelude::vec::Vec;


impl DdcBucket {

    pub fn message_get_v_nodes_by_cluster(&self, cluster_id: ClusterId) -> Vec<VNodeToken> {
        self.topology_store.get_v_nodes_by_cluster(cluster_id)
    }

    pub fn message_get_v_nodes_by_node(&self, node_key: NodeKey) -> Vec<VNodeToken> {
        self.topology_store.get_v_nodes_by_node(node_key)
    }

    pub fn message_get_node_by_v_node(&self, cluster_id: ClusterId, v_node: VNodeToken) -> Result<NodeKey> {
        self.topology_store.get_node_by_v_node(cluster_id, v_node)
    }
    
}