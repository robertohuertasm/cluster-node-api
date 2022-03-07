use crate::domain::{
    operation::Operation,
    repository::{OperationRepository, RepositoryError, RepositoryResult},
};
use async_trait::async_trait;
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
        let result = sqlx::query_as::<_, Operation>(
            r#"
        INSERT INTO operations (id, operation_type, node_id)
        VALUES ($1, $2, $3)
        RETURNING id, operation_type, node_id, created_at, updated_at
        "#,
        )
        .bind(&operation.id)
        .bind(&operation.operation_type)
        .bind(&operation.node_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            RepositoryError::AlreadyExists
        })
    }
}
