use crate::domain::{
    models::{Node, Operation, OperationType},
    repository::{NodeRepository, RepositoryError},
};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OperationServiceError {
    #[error("Node not found: `{0}`")]
    NodeNotFound(Uuid),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError),
}

pub type OperationServiceResult = Result<Operation, OperationServiceError>;

#[derive(Debug, Clone)]
pub struct OperationService<N: NodeRepository> {
    node_repository: N,
}

impl<N> OperationService<N>
where
    N: NodeRepository,
{
    pub fn new(node_repository: N) -> Self {
        Self { node_repository }
    }

    #[instrument(skip(self))]
    pub async fn power_on(&self, node_id: &Uuid) -> OperationServiceResult {
        self.create_operation(node_id, OperationType::PowerOn).await
    }

    #[instrument(skip(self))]
    pub async fn power_off(&self, node_id: &Uuid) -> OperationServiceResult {
        self.create_operation(node_id, OperationType::PowerOff)
            .await
    }

    #[instrument(skip(self))]
    pub async fn reboot(&self, node_id: &Uuid) -> OperationServiceResult {
        self.create_operation(node_id, OperationType::Reboot).await
    }

    #[instrument(skip(self))]
    async fn create_operation(
        &self,
        node_id: &Uuid,
        operation_type: OperationType,
    ) -> OperationServiceResult {
        self.node_check(node_id).await?;
        let operation = Operation::new(node_id.to_owned(), operation_type);
        let operation = self.node_repository.create_operation(&operation).await?;
        Ok(operation)
    }

    #[instrument(skip(self))]
    async fn node_check(&self, node_id: &Uuid) -> Result<Node, OperationServiceError> {
        let result = self.node_repository.get_node(node_id).await;
        result.map_err(|e| {
            tracing::error!("Node not found in database: {:?}", e);
            OperationServiceError::NodeNotFound(node_id.to_owned())
        })
    }
}
