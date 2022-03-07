use crate::domain::{
    models::{Node, NodeStatus, Operation, OperationType},
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
        let mut node = self.node_check(node_id).await?;
        // TODO: ideally, this should be transactional
        let operation = Operation::new(node_id.to_owned(), operation_type);
        let operation = self
            .operation_repository
            .create_operation(&operation)
            .await?;

        node.status = match operation_type {
            OperationType::PowerOn => NodeStatus::PowerOn,
            OperationType::PowerOff => NodeStatus::PowerOff,
            OperationType::Reboot => NodeStatus::Rebooting,
        };
        self.node_repository.update_node(&node).await?;
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
