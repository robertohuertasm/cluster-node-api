use super::RepositoryResult;
use crate::domain::node::Node;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NodeRepository: Send + Sync + 'static {
    async fn get_nodes(&self) -> RepositoryResult<Vec<Node>>;
    async fn get_node(&self, node_id: &Uuid) -> RepositoryResult<Node>;
    async fn create_node(&self, node: &Node) -> RepositoryResult<Node>;
    async fn update_node(&self, node: &Node) -> RepositoryResult<Node>;
    async fn delete_node(&self, node_id: &Uuid) -> RepositoryResult<Uuid>;
}
