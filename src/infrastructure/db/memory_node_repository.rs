use crate::domain::{
    node::Node,
    repository::{
        node_repository::{NodeFilter, NodeRepository},
        RepositoryError, RepositoryResult,
    },
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

pub struct MemoryNodeRepository {
    nodes: RwLock<Vec<Node>>,
}

impl Default for MemoryNodeRepository {
    fn default() -> Self {
        Self {
            nodes: RwLock::new(vec![]),
        }
    }
}

#[async_trait]
impl NodeRepository for MemoryNodeRepository {
    #[instrument(skip(self))]
    async fn get_nodes(&self, _filter: Option<NodeFilter>) -> RepositoryResult<Vec<Node>> {
        // TODO: implement filter would require a refactor on how we store things in memory.
        // skipping filters for the time being as MemoryNodeRepository is not used and it's only for the sake of the example
        // of how to build another implementation of the NodeRepository trait.
        let nodes = self.nodes.read()?;
        Ok(nodes.iter().cloned().collect())
    }

    #[instrument(skip(self))]
    async fn get_node(&self, node_id: &uuid::Uuid) -> RepositoryResult<Node> {
        let nodes = self.nodes.read()?;
        let result = nodes
            .iter()
            .find(|u| &u.id == node_id)
            .cloned()
            .ok_or_else(|| RepositoryError::InvalidId);

        if result.is_err() {
            tracing::error!("Couldn't retrive a node with id {}", node_id);
        }

        result
    }

    #[instrument(skip(self))]
    async fn create_node(&self, node: &Node) -> RepositoryResult<Node> {
        if self.get_node(&node.id).await.is_ok() {
            tracing::error!("node with id {} already exists", node.id);
            return Err(RepositoryError::AlreadyExists);
        }
        let mut new_node = node.to_owned();
        new_node.created_at = Some(Utc::now());
        let mut nodes = self.nodes.write().unwrap();
        nodes.push(new_node.clone());
        tracing::trace!("node with id {} correctly created", node.id);
        Ok(new_node)
    }

    #[instrument(skip(self))]
    async fn update_node(&self, node: &Node) -> RepositoryResult<Node> {
        if let Ok(old_node) = self.get_node(&node.id).await {
            let mut updated_node = node.to_owned();
            updated_node.created_at = old_node.created_at;
            updated_node.updated_at = Some(Utc::now());
            let mut nodes = self.nodes.write().unwrap();
            nodes.retain(|x| x.id != node.id);
            nodes.push(updated_node.clone());
            tracing::debug!("Node with id {} correctly updated", node.id);
            Ok(updated_node)
        } else {
            tracing::error!("Node {} does not exit", node.id);
            Err(RepositoryError::DoesNotExist)
        }
    }

    #[instrument(skip(self), err)]
    async fn delete_node(&self, node_id: &Uuid) -> RepositoryResult<Uuid> {
        let mut nodes = self.nodes.write()?;
        nodes.retain(|x| &x.id != node_id);
        Ok(node_id.to_owned())
    }
}
