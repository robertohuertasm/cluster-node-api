use crate::{
    domain::{
        operation::{Operation, OperationType},
        repository::{NodeRepository, OperationRepository},
    },
    infrastructure::auth,
};
use actix_web::{
    web::{self, PathConfig},
    HttpResponse,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use web::ServiceConfig;

use super::path_config_handler;

const PATH: &str = "/v1/operations";

// TODO: CHANGE NAME TO THIS TO AVOID CONFUSION: USE CONTROLLER OR SOMETHING?
pub fn service<N: NodeRepository, R: OperationRepository>(cfg: &mut ServiceConfig) {
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OperationDTO {
    pub id: Uuid,
    pub node_id: Uuid,
}

#[instrument(skip(repo))]
async fn post_poweron<N: NodeRepository, R: OperationRepository>(
    operation: web::Json<OperationDTO>,
    repo: web::Data<R>,
) -> HttpResponse {
    let operation = Operation::new(operation.node_id, OperationType::PowerOn);
    match repo.create_operation(&operation).await {
        Ok(operation) => HttpResponse::Created().json(operation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn post_poweroff<N: NodeRepository, R: OperationRepository>(
    operation: web::Json<OperationDTO>,
    repo: web::Data<R>,
) -> HttpResponse {
    let operation = Operation::new(operation.node_id, OperationType::PowerOff);

    match repo.create_operation(&operation).await {
        Ok(operation) => HttpResponse::Created().json(operation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn post_reboot<N: NodeRepository, R: OperationRepository>(
    operation: web::Json<OperationDTO>,
    repo: web::Data<R>,
) -> HttpResponse {
    let operation = Operation::new(operation.node_id, OperationType::Reboot);
    match repo.create_operation(&operation).await {
        Ok(operation) => HttpResponse::Created().json(operation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}
