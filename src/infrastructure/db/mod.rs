mod memory_cluster_repository;
mod memory_node_repository;
mod postgres_cluster_repository;
mod postgres_node_repository;
mod postgres_operation_repository;

pub use memory_cluster_repository::MemoryClusterRepository;
pub use memory_node_repository::MemoryNodeRepository;
pub use postgres_cluster_repository::PostgresClusterRepository;
pub use postgres_node_repository::PostgresNodeRepository;
pub use postgres_operation_repository::PostgresOperationRepository;

use crate::domain::repository::RepositoryError;

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        RepositoryError::Generic(Box::new(error))
    }
}
