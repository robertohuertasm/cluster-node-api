use super::RepositoryResult;
use crate::domain::cluster::Cluster;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ClusterRepository: Send + Sync + 'static {
    async fn get_clusters(&self) -> RepositoryResult<Vec<Cluster>>;
    async fn get_cluster(&self, cluster_id: &Uuid) -> RepositoryResult<Cluster>;
    async fn create_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster>;
    async fn update_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster>;
    async fn delete_cluster(&self, cluster_id: &Uuid) -> RepositoryResult<Uuid>;
}
