use crate::{
    application::operation_service::{OperationService, OperationServiceResult},
    domain::repository::{NodeRepository, OperationRepository},
    infrastructure::auth,
};
use actix_web::{
    web::{self, PathConfig},
    HttpResponse,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use tracing::instrument;
use uuid::Uuid;
use web::ServiceConfig;

use super::path_config_handler;

const PATH: &str = "/v1/operations";

pub fn configuration<N, R>(cfg: &mut ServiceConfig)
where
    N: NodeRepository,
    R: OperationRepository,
{
    cfg.service(
        web::scope(PATH)
            .wrap(HttpAuthentication::bearer(auth::validator))
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // POST
            .route("/poweron", web::post().to(post_poweron::<N, R>))
            .route("/poweroff", web::post().to(post_poweroff::<N, R>))
            .route("/reboot", web::post().to(post_reboot::<N, R>)),
    );
}

fn to_response(operation_result: OperationServiceResult) -> HttpResponse {
    match operation_result {
        Ok(operation) => HttpResponse::Created().json(operation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(svc))]
async fn post_poweron<N: NodeRepository, R: OperationRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<N, R>>,
) -> HttpResponse {
    to_response(svc.power_on(&node_id).await)
}

#[instrument(skip(svc))]
async fn post_poweroff<N: NodeRepository, R: OperationRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<N, R>>,
) -> HttpResponse {
    to_response(svc.power_off(&node_id).await)
}

#[instrument(skip(svc))]
async fn post_reboot<N: NodeRepository, R: OperationRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<N, R>>,
) -> HttpResponse {
    to_response(svc.reboot(&node_id).await)
}

#[cfg(test)]
mod tests {

    use crate::domain::{
        models::{Node, NodeStatus, Operation, OperationType},
        repository::{
            node_repository::MockNodeRepository, operation_repository::MockOperationRepository,
            RepositoryError,
        },
    };

    use super::*;
    use actix_web::{body::MessageBody, http::StatusCode};
    use chrono::Utc;

    fn create_test_node(id: uuid::Uuid, name: String) -> Node {
        Node {
            id,
            name,
            cluster_id: uuid::Uuid::new_v4(),
            status: NodeStatus::PowerOn,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    fn prepare_operation_svc() -> OperationService<MockNodeRepository, MockOperationRepository> {
        let mut node_repo = MockNodeRepository::default();
        node_repo.expect_get_node().once().returning(move |id| {
            let node = create_test_node(*id, "my_node".to_string());
            Ok(node)
        });

        node_repo
            .expect_update_node()
            .once()
            .returning(|node| Ok(node.clone()));

        let mut ops_repo = MockOperationRepository::default();
        ops_repo
            .expect_create_operation()
            .returning(|op| Ok(op.clone()));

        OperationService::new(node_repo, ops_repo)
    }

    fn prepare_operation_svc_with_error(
    ) -> OperationService<MockNodeRepository, MockOperationRepository> {
        let mut node_repo = MockNodeRepository::default();
        node_repo
            .expect_get_node()
            .once()
            .returning(|_| Err(RepositoryError::DoesNotExist));

        let ops_repo = MockOperationRepository::default();

        OperationService::new(node_repo, ops_repo)
    }

    #[actix_rt::test]
    async fn poweron_works() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc();
        let res = post_poweron(web::Json(node_id), web::Data::new(svc)).await;

        let body = res.into_body().try_into_bytes().unwrap();
        let operation = serde_json::from_slice::<'_, Operation>(&body).ok().unwrap();

        assert_eq!(operation.node_id, node_id);
        assert_eq!(operation.operation_type, OperationType::PowerOn);
    }

    #[actix_rt::test]
    async fn poweron_errors_properly() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc_with_error();
        let res = post_poweron(web::Json(node_id), web::Data::new(svc)).await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_rt::test]
    async fn poweroff_works() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc();
        let res = post_poweroff(web::Json(node_id), web::Data::new(svc)).await;

        let body = res.into_body().try_into_bytes().unwrap();
        let operation = serde_json::from_slice::<'_, Operation>(&body).ok().unwrap();

        assert_eq!(operation.node_id, node_id);
        assert_eq!(operation.operation_type, OperationType::PowerOff);
    }

    #[actix_rt::test]
    async fn poweroff_errors_properly() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc_with_error();
        let res = post_poweroff(web::Json(node_id), web::Data::new(svc)).await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_rt::test]
    async fn reboot_works() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc();
        let res = post_reboot(web::Json(node_id), web::Data::new(svc)).await;

        let body = res.into_body().try_into_bytes().unwrap();
        let operation = serde_json::from_slice::<'_, Operation>(&body).ok().unwrap();

        assert_eq!(operation.node_id, node_id);
        assert_eq!(operation.operation_type, OperationType::Reboot);
    }

    #[actix_rt::test]
    async fn reboot_errors_properly() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc_with_error();
        let res = post_reboot(web::Json(node_id), web::Data::new(svc)).await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
