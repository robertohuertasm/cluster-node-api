mod entities;
mod postgres_cluster_repository;
mod postgres_node_repository;

pub use postgres_cluster_repository::PostgresClusterRepository;
pub use postgres_node_repository::PostgresNodeRepository;

use crate::domain::repository::RepositoryError;

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        RepositoryError::Generic(Box::new(error))
    }
}
