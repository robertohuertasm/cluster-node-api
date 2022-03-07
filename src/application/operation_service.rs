use crate::domain::{
    models::{Operation, OperationType},
    repository::{NodeRepository, OperationRepository, RepositoryError},
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
pub struct OperationService<N: NodeRepository, O: OperationRepository> {
    node_repository: N,
    operation_repository: O,
}

impl<N, O> OperationService<N, O>
where
    N: NodeRepository,
    O: OperationRepository,
{
    pub fn new(node_repository: N, operation_repository: O) -> Self {
        Self {
            node_repository,
            operation_repository,
        }
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
        let operation = self
            .operation_repository
            .create_operation(&operation)
            .await?;
        Ok(operation)
    }

    #[instrument(skip(self))]
    async fn node_check(&self, node_id: &Uuid) -> Result<(), OperationServiceError> {
        if let Err(e) = self.node_repository.get_node(node_id).await {
            tracing::error!("Node not found in database: {:?}", e);
            return Err(OperationServiceError::NodeNotFound(node_id.to_owned()));
        }
        Ok(())
    }
}
