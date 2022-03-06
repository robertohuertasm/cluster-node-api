use crate::domain::{
    node::Node,
    repository::{NodeRepository, RepositoryError, RepositoryResult},
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

pub struct PostgresNodeRepository {
    pool: sqlx::PgPool,
}

impl PostgresNodeRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NodeRepository for PostgresNodeRepository {
    #[instrument(skip(self))]
    async fn get_nodes(&self) -> RepositoryResult<Vec<Node>> {
        let result = sqlx::query_as::<_, Node>(
            "SELECT id, name, cluster_id, created_at, updated_at FROM nodes",
        )
        .fetch_all(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::Generic(Box::new(e))
        })
    }

    #[instrument(skip(self))]
    async fn get_node(&self, node_id: &uuid::Uuid) -> RepositoryResult<Node> {
        let result = sqlx::query_as::<_, Node>(
            "SELECT id, name, cluster_id, created_at, updated_at FROM nodes WHERE id = $1",
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::InvalidId
        })
    }

    #[instrument(skip(self))]
    async fn create_node(&self, node: &Node) -> RepositoryResult<Node> {
        let result = sqlx::query_as::<_, Node>(
            r#"
        INSERT INTO nodes (id, name, cluster_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.id)
        .bind(&node.name)
        .bind(&node.cluster_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::AlreadyExists
        })
    }

    #[instrument(skip(self))]
    async fn update_node(&self, node: &Node) -> RepositoryResult<Node> {
        let result = sqlx::query_as::<_, Node>(
            r#"
            UPDATE nodes
            SET name = $1, updated_at = $2, cluster_id = $3
            WHERE id = $4
            RETURNING id, name, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.name)
        .bind(Utc::now())
        .bind(&node.cluster_id)
        .bind(&node.id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::DoesNotExist
        })
    }

    #[instrument(skip(self), err)]
    async fn delete_node(&self, node_id: &Uuid) -> RepositoryResult<Uuid> {
        let result = sqlx::query_as::<_, Node>(
            r#"
            DELETE FROM nodes
            WHERE id = $1
            RETURNING id, name, cluster_id, created_at, updated_at
        "#,
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await;

        result.map(|u| u.id).map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::DoesNotExist
        })
    }
}
