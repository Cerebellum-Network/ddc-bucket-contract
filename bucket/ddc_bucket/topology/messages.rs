//! The public interface to manage Nodes.

use crate::ddc_bucket::{ClusterId, DdcBucket, Error::*, NodeKey, Result, VNodeToken};
use ink_prelude::vec::Vec;

impl DdcBucket {
    pub fn message_get_v_nodes_by_cluster(&self, cluster_id: ClusterId) -> Vec<VNodeToken> {
        self.topology.get_v_nodes_by_cluster(cluster_id)
    }

    pub fn message_get_v_nodes_by_node(
        &self,
        cluster_id: ClusterId,
        node_key: NodeKey,
    ) -> Result<Vec<VNodeToken>> {
        let node = self.nodes.get(node_key)?;

        if node.cluster_id != Some(cluster_id) {
            return Err(NodeIsNotAddedToCluster(cluster_id));
        }

        Ok(self.topology.get_v_nodes_by_node(node_key))
    }

    pub fn message_get_node_by_v_node(
        &self,
        cluster_id: ClusterId,
        v_node: VNodeToken,
    ) -> Result<NodeKey> {
        self.topology.get_node_by_v_node(cluster_id, v_node)
    }
}
