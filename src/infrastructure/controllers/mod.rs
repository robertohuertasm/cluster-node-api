use tracing::instrument;

pub mod clusters;
pub mod features;
pub mod health;
pub mod nodes;

#[instrument(fields( path=?_req.path()), skip(_req))]
fn path_config_handler(
    err: actix_web::error::PathError,
    _req: &actix_web::HttpRequest,
) -> actix_web::Error {
    tracing::error!(error=?err, "There was an error with the path");
    actix_web::error::ErrorBadRequest(err)
}
