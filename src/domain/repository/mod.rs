pub mod cluster_repository;
pub mod node_repository;
mod repository_error;

pub use cluster_repository::ClusterRepository;
pub use node_repository::NodeRepository;
pub use repository_error::RepositoryError;

pub type RepositoryResult<T> = Result<T, RepositoryError>;
