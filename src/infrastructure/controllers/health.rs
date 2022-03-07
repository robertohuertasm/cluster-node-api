use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};
use tracing::instrument;

#[instrument(skip(cfg), level = "trace")]
pub fn configuration(cfg: &mut ServiceConfig) {
    tracing::trace!("Init health service");
    cfg.route("/health", web::get().to(health_check));
}

#[instrument]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, App};

    #[actix_rt::test]
    async fn health_check_works() {
        let res = health_check().await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn health_check_integration_works() {
        let app = App::new().configure(configuration);
        let mut app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/health")
            .to_request();
        let res = actix_web::test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
    }
}
