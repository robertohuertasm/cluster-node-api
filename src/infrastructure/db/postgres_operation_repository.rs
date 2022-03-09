use crate::domain::{
    models::{NodeStatus, Operation, OperationType},
    repository::{OperationRepository, RepositoryResult},
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;

pub struct PostgresOperationRepository {
    pool: sqlx::PgPool,
}

impl PostgresOperationRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl Clone for PostgresOperationRepository {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[async_trait]
impl OperationRepository for PostgresOperationRepository {
    #[instrument(skip(self))]
    async fn create_operation(&self, operation: &Operation) -> RepositoryResult<Operation> {
        let node_status = match operation.operation_type {
            OperationType::PowerOn => NodeStatus::PowerOn,
            OperationType::PowerOff => NodeStatus::PowerOff,
            OperationType::Reboot => NodeStatus::Rebooting,
        };

        let mut tx = self.pool.begin().await?;

        let insert_op = sqlx::query_as::<_, Operation>(
            r#"
        INSERT INTO operations (id, operation_type, node_id)
        VALUES ($1, $2, $3)
        RETURNING id, operation_type, node_id, created_at, updated_at
        "#,
        )
        .bind(&operation.id)
        .bind(&operation.operation_type)
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
                Ok(o)
            }
            Err(e) => {
                tracing::error!("Error creating operation: {:?}", e);
                Err(e.into())
            }
        }
    }
}
