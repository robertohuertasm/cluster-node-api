use crate::domain::{
    models::Node,
    repository::{node_repository::NodeFilter, NodeRepository, RepositoryError, RepositoryResult},
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

impl Clone for PostgresNodeRepository {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[async_trait]
impl NodeRepository for PostgresNodeRepository {
    #[instrument(skip(self))]
    async fn get_nodes(&self, filter: Option<NodeFilter>) -> RepositoryResult<Vec<Node>> {
        let query = if let Some(filter) = filter {
            tracing::error!("FILTER {:?}", filter);
            sqlx::query_as::<_, Node>(
                r"
                SELECT n.id, n.name, n.status, n.cluster_id, n.created_at, n.updated_at
                FROM nodes n
                JOIN clusters c on n.cluster_id = c.id
                where n.name like $1 or c.name like $1;
                ",
            )
            .bind(format!("%{}%", filter.name))
        } else {
            sqlx::query_as::<_, Node>(
                "SELECT id, name, status, cluster_id, created_at, updated_at FROM nodes",
            )
        };

        let result = query.fetch_all(&self.pool).await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::Generic(Box::new(e))
        })
    }

    #[instrument(skip(self))]
    async fn get_node(&self, node_id: &uuid::Uuid) -> RepositoryResult<Node> {
        let result = sqlx::query_as::<_, Node>(
            "SELECT id, name, status, cluster_id, created_at, updated_at FROM nodes WHERE id = $1",
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
        INSERT INTO nodes (id, name, status, cluster_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, status, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.id)
        .bind(&node.name)
        .bind(&node.status)
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
            SET name = $1, status = $2, cluster_id = $3, updated_at = $4
            WHERE id = $5
            RETURNING id, name, status, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.name)
        .bind(&node.status)
        .bind(&node.cluster_id)
        .bind(Utc::now())
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
            RETURNING id, name, status, cluster_id, created_at, updated_at
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
