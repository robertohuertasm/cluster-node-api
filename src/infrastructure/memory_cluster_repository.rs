use crate::domain::{
    cluster::Cluster,
    repository::{cluster_repository::ClusterRepository, RepositoryError, RepositoryResult},
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

pub struct MemoryClusterRepository {
    clusters: RwLock<Vec<Cluster>>,
}

impl Default for MemoryClusterRepository {
    fn default() -> Self {
        Self {
            clusters: RwLock::new(vec![]),
        }
    }
}

#[async_trait]
impl ClusterRepository for MemoryClusterRepository {
    #[instrument(skip(self))]
    async fn get_clusters(&self) -> RepositoryResult<Vec<Cluster>> {
        let clusters = self.clusters.read()?;
        Ok(clusters.iter().cloned().collect())
    }

    #[instrument(skip(self))]
    async fn get_cluster(&self, cluster_id: &uuid::Uuid) -> RepositoryResult<Cluster> {
        let clusters = self.clusters.read()?;
        let result = clusters
            .iter()
            .find(|u| &u.id == cluster_id)
            .cloned()
            .ok_or_else(|| RepositoryError::InvalidId);

        if result.is_err() {
            tracing::error!("Couldn't retrive a cluster with id {}", cluster_id);
        }

        result
    }

    #[instrument(skip(self))]
    async fn create_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster> {
        if self.get_cluster(&cluster.id).await.is_ok() {
            tracing::error!("cluster with id {} already exists", cluster.id);
            return Err(RepositoryError::AlreadyExists);
        }
        let mut new_cluster = cluster.to_owned();
        new_cluster.created_at = Some(Utc::now());
        let mut clusters = self.clusters.write().unwrap();
        clusters.push(new_cluster.clone());
        tracing::trace!("cluster with id {} correctly created", cluster.id);
        Ok(new_cluster)
    }

    #[instrument(skip(self))]
    async fn update_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster> {
        if let Ok(old_cluster) = self.get_cluster(&cluster.id).await {
            let mut updated_cluster = cluster.to_owned();
            updated_cluster.created_at = old_cluster.created_at;
            updated_cluster.updated_at = Some(Utc::now());
            let mut clusters = self.clusters.write().unwrap();
            clusters.retain(|x| x.id != cluster.id);
            clusters.push(updated_cluster.clone());
            tracing::debug!("Cluster with id {} correctly updated", cluster.id);
            Ok(updated_cluster)
        } else {
            tracing::error!("Cluster {} does not exit", cluster.id);
            Err(RepositoryError::DoesNotExist)
        }
    }

    #[instrument(skip(self), err)]
    async fn delete_cluster(&self, cluster_id: &Uuid) -> RepositoryResult<Uuid> {
        let mut clusters = self.clusters.write()?;
        clusters.retain(|x| &x.id != cluster_id);
        Ok(cluster_id.to_owned())
    }
}
