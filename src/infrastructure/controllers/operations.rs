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

// TODO: CHANGE NAME TO THIS TO AVOID CONFUSION: USE CONTROLLER OR SOMETHING?
pub fn service<N, R>(cfg: &mut ServiceConfig)
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
