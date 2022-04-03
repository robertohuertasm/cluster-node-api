use crate::{
    domain::{
        models::{Node, Operation, OperationType},
        repository::{
            node_repository::NodeFilter, NodeRepository, RepositoryError, RepositoryResult,
        },
    },
    infrastructure::db::entities::DbNode,
    infrastructure::db::entities::DbOperation,
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

use super::entities::{DbNodeStatus, DbOperationType};

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
            sqlx::query_as::<_, DbNode>(
                r"
                SELECT n.id, n.name, n.status, n.cluster_id, n.created_at, n.updated_at
                FROM nodes n
                JOIN clusters c on n.cluster_id = c.id
                where n.name like $1 or c.name like $1;
                ",
            )
            .bind(format!("%{}%", filter.name))
        } else {
            sqlx::query_as::<_, DbNode>(
                "SELECT id, name, status, cluster_id, created_at, updated_at FROM nodes",
            )
        };

        let result = query.fetch_all(&self.pool).await;

        result
            .map(|x| x.into_iter().map(|x| x.into()).collect())
            .map_err(|e| {
                tracing::error!("{:?}", e);
                e.into()
            })
    }

    #[instrument(skip(self))]
    async fn get_node(&self, node_id: &uuid::Uuid) -> RepositoryResult<Node> {
        let result = sqlx::query_as::<_, DbNode>(
            "SELECT id, name, status, cluster_id, created_at, updated_at FROM nodes WHERE id = $1",
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await;

        result.map(|x| x.into()).map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::InvalidId
        })
    }

    #[instrument(skip(self))]
    async fn create_node(&self, node: &Node) -> RepositoryResult<Node> {
        let db_status: DbNodeStatus = node.status.into();
        let result = sqlx::query_as::<_, DbNode>(
            r#"
        INSERT INTO nodes (id, name, status, cluster_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, status, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.id)
        .bind(&node.name)
        .bind(db_status)
        .bind(&node.cluster_id)
        .fetch_one(&self.pool)
        .await;

        result.map(|x| x.into()).map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::AlreadyExists
        })
    }

    #[instrument(skip(self))]
    async fn update_node(&self, node: &Node) -> RepositoryResult<Node> {
        let db_status: DbNodeStatus = node.status.into();
        let result = sqlx::query_as::<_, DbNode>(
            r#"
            UPDATE nodes
            SET name = $1, status = $2, cluster_id = $3, updated_at = $4
            WHERE id = $5
            RETURNING id, name, status, cluster_id, created_at, updated_at
        "#,
        )
        .bind(&node.name)
        .bind(db_status)
        .bind(&node.cluster_id)
        .bind(Utc::now())
        .bind(&node.id)
        .fetch_one(&self.pool)
        .await;

        result.map(|x| x.into()).map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::DoesNotExist
        })
    }

    #[instrument(skip(self), err)]
    async fn delete_node(&self, node_id: &Uuid) -> RepositoryResult<Uuid> {
        let result = sqlx::query_as::<_, DbNode>(
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

    #[instrument(skip(self))]
    async fn create_operation(&self, operation: &Operation) -> RepositoryResult<Operation> {
        let node_status = match operation.operation_type {
            OperationType::PowerOn => DbNodeStatus::PowerOn,
            OperationType::PowerOff => DbNodeStatus::PowerOff,
            OperationType::Reboot => DbNodeStatus::Rebooting,
        };

        let db_opt_type: DbOperationType = operation.operation_type.into();

        let mut tx = self.pool.begin().await?;

        let insert_op = sqlx::query_as::<_, DbOperation>(
            r#"
        INSERT INTO operations (id, operation_type, node_id)
        VALUES ($1, $2, $3)
        RETURNING id, operation_type, node_id, created_at, updated_at
        "#,
        )
        .bind(&operation.id)
        .bind(db_opt_type)
        .bind(&operation.node_id)
        .fetch_one(&mut tx)
        .await;

        match insert_op {
            Ok(o) => {
                if let Err(e) = sqlx::query(
                    r#"
                    UPDATE nodes
                    SET  status = $1, updated_at = $2
                    WHERE id = $3
                "#,
                )
                .bind(node_status)
                .bind(Utc::now())
                .bind(&operation.node_id)
                .execute(&mut tx)
                .await
                {
                    tracing::error!("Error updating node while creating operation: {:?}", e);
                    return Err(e.into());
                }
                tx.commit().await?;
                Ok(o.into())
            }
            Err(e) => {
                tracing::error!("Error creating operation: {:?}", e);
                Err(e.into())
            }
        }
    }
}
