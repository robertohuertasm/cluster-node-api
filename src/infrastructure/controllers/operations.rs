use crate::{
    application::operation_service::{OperationService, OperationServiceResult},
    domain::repository::NodeRepository,
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

pub fn configuration<R: NodeRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .wrap(HttpAuthentication::bearer(auth::validator))
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // POST
            .route("/poweron", web::post().to(post_poweron::<R>))
            .route("/poweroff", web::post().to(post_poweroff::<R>))
            .route("/reboot", web::post().to(post_reboot::<R>)),
    );
}

fn to_response(operation_result: OperationServiceResult) -> HttpResponse {
    match operation_result {
        Ok(operation) => HttpResponse::Created().json(operation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(svc))]
async fn post_poweron<R: NodeRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<R>>,
) -> HttpResponse {
    to_response(svc.power_on(&node_id).await)
}

#[instrument(skip(svc))]
async fn post_poweroff<R: NodeRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<R>>,
) -> HttpResponse {
    to_response(svc.power_off(&node_id).await)
}

#[instrument(skip(svc))]
async fn post_reboot<R: NodeRepository>(
    node_id: web::Json<Uuid>,
    svc: web::Data<OperationService<R>>,
) -> HttpResponse {
    let r = svc.reboot(&node_id).await;
    // start a new thread to simulate poweron in a few seconds
    actix_web::rt::spawn(async move {
        actix_web::rt::time::sleep(std::time::Duration::from_secs(5)).await;
        if let Err(e) = svc.power_on_without_operation(&node_id).await {
            tracing::error!("Error powering on after rebooting: {:?}", e);
        }
    });
    to_response(r)
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::domain::{
        models::{Node, NodeStatus, Operation, OperationType},
        repository::{node_repository::MockNodeRepository, RepositoryError},
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

    fn prepare_operation_svc() -> OperationService<MockNodeRepository> {
        let mut node_repo = MockNodeRepository::default();
        node_repo.expect_get_node().returning(move |id| {
            let node = create_test_node(*id, "my_node".to_string());
            Ok(node)
        });

        node_repo
            .expect_update_node()
            .returning(|node| Ok(node.clone()));

        node_repo
            .expect_create_operation()
            .returning(|op| Ok(op.clone()));

        OperationService::new(node_repo)
    }

    fn prepare_operation_svc_with_error() -> OperationService<MockNodeRepository> {
        let mut node_repo = MockNodeRepository::default();
        node_repo
            .expect_get_node()
            .once()
            .returning(|_| Err(RepositoryError::DoesNotExist));

        OperationService::new(node_repo)
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
        let mut node_repo = MockNodeRepository::default();

        let node_id = uuid::Uuid::new_v4();

        node_repo.expect_get_node().times(2).returning(move |id| {
            let node = create_test_node(*id, "my_node".to_string());
            Ok(node)
        });

        node_repo
            .expect_update_node()
            .once()
            .returning(|node| Ok(node.clone()));

        node_repo
            .expect_create_operation()
            .once()
            .returning(|op| Ok(op.clone()));

        let svc = OperationService::new(node_repo);
        let res = post_reboot(web::Json(node_id), web::Data::new(svc)).await;

        let body = res.into_body().try_into_bytes().unwrap();
        let operation = serde_json::from_slice::<'_, Operation>(&body).ok().unwrap();

        assert_eq!(operation.node_id, node_id);
        assert_eq!(operation.operation_type, OperationType::Reboot);

        // waiting >5 secs to check that the node is set to power on
        actix_rt::time::sleep(Duration::from_secs(6)).await;
    }

    #[actix_rt::test]
    async fn reboot_errors_properly() {
        let node_id = uuid::Uuid::new_v4();
        let svc = prepare_operation_svc_with_error();
        let res = post_reboot(web::Json(node_id), web::Data::new(svc)).await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
