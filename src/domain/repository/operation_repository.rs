use super::RepositoryResult;
use crate::domain::operation::Operation;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct NodeFilter {
    pub name: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OperationRepository: Send + Sync + 'static {
    async fn create_operation(&self, operation: &Operation) -> RepositoryResult<Operation>;
}
