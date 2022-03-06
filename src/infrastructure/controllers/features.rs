use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};
use tracing::instrument;

#[instrument(skip(cfg), level = "trace")]
pub fn service(cfg: &mut ServiceConfig) {
    cfg.route("/features", web::get().to(features));
}

#[instrument]
async fn features() -> HttpResponse {
    HttpResponse::Ok().body("WTF")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, App};

    #[actix_rt::test]
    async fn health_check_works() {
        let res = features().await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let data = res
            .headers()
            .get("thread-id")
            .map(|h| h.to_str().ok())
            .flatten();
        assert_eq!(data, Some("5"));
    }

    #[actix_rt::test]
    async fn health_check_integration_works() {
        let app = App::new().configure(service);
        let mut app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/features")
            .to_request();
        let res = actix_web::test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let data = res
            .headers()
            .get("thread-id")
            .map(|h| h.to_str().ok())
            .flatten();
        assert_eq!(data, Some("5"));
    }
}
