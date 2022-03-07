use crate::domain::{
    cluster::Cluster,
    repository::{ClusterRepository, RepositoryError, RepositoryResult},
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

pub struct PostgresClusterRepository {
    pool: sqlx::PgPool,
}

impl PostgresClusterRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl Clone for PostgresClusterRepository {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[async_trait]
impl ClusterRepository for PostgresClusterRepository {
    #[instrument(skip(self))]
    async fn get_clusters(&self) -> RepositoryResult<Vec<Cluster>> {
        let result =
            sqlx::query_as::<_, Cluster>("SELECT id, name, created_at, updated_at FROM clusters")
                .fetch_all(&self.pool)
                .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::Generic(Box::new(e))
        })
    }

    #[instrument(skip(self))]
    async fn get_cluster(&self, cluster_id: &uuid::Uuid) -> RepositoryResult<Cluster> {
        let result = sqlx::query_as::<_, Cluster>(
            "SELECT id, name, created_at, updated_at FROM clusters WHERE id = $1",
        )
        .bind(cluster_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::InvalidId
        })
    }

    #[instrument(skip(self))]
    async fn create_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster> {
        let result = sqlx::query_as::<_, Cluster>(
            r#"
        INSERT INTO clusters (id, name)
        VALUES ($1, $2)
        RETURNING id, name, created_at, updated_at
        "#,
        )
        .bind(&cluster.id)
        .bind(&cluster.name)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::AlreadyExists
        })
    }

    #[instrument(skip(self))]
    async fn update_cluster(&self, cluster: &Cluster) -> RepositoryResult<Cluster> {
        let result = sqlx::query_as::<_, Cluster>(
            r#"
            UPDATE clusters
            SET name = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, created_at, updated_at
        "#,
        )
        .bind(&cluster.name)
        .bind(Utc::now())
        .bind(&cluster.id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::DoesNotExist
        })
    }

    #[instrument(skip(self), err)]
    async fn delete_cluster(&self, cluster_id: &Uuid) -> RepositoryResult<Uuid> {
        let result = sqlx::query_as::<_, Cluster>(
            r#"
            DELETE FROM clusters
            WHERE id = $1
            RETURNING id, name, created_at, updated_at
        "#,
        )
        .bind(cluster_id)
        .fetch_one(&self.pool)
        .await;

        result.map(|u| u.id).map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::DoesNotExist
        })
    }
}
