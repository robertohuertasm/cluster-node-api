use super::RepositoryResult;
use crate::domain::models::{Node, Operation};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct NodeFilter {
    pub name: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NodeRepository: Send + Sync + 'static {
    async fn get_nodes(&self, name: Option<NodeFilter>) -> RepositoryResult<Vec<Node>>;
    async fn get_node(&self, node_id: &Uuid) -> RepositoryResult<Node>;
    async fn create_node(&self, node: &Node) -> RepositoryResult<Node>;
    async fn update_node(&self, node: &Node) -> RepositoryResult<Node>;
    async fn delete_node(&self, node_id: &Uuid) -> RepositoryResult<Uuid>;
    async fn create_operation(&self, operation: &Operation) -> RepositoryResult<Operation>;
}
