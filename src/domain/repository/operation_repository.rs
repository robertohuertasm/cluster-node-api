use super::RepositoryResult;
use crate::domain::models::Operation;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OperationRepository: Send + Sync + 'static {
    async fn create_operation(&self, operation: &Operation) -> RepositoryResult<Operation>;
}
