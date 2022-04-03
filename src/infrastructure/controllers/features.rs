use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};
use tracing::instrument;

const FEAT_1: &str = "Search by node name or cluster name";
const FEAT_2: &str = "Manage clusters";
const FEAT_3: &str = "Manage nodes";
const FEAT_4: &str = "Create node commands";

#[instrument(skip(cfg), level = "trace")]
pub fn configuration(cfg: &mut ServiceConfig) {
    cfg.route("/v1/features", web::get().to(features));
}

#[instrument]
async fn features() -> HttpResponse {
    HttpResponse::Ok().json(vec![FEAT_1, FEAT_2, FEAT_3, FEAT_4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::MessageBody, http::StatusCode, App};

    #[actix_rt::test]
    async fn features_works() {
        let res = features().await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().try_into_bytes().unwrap();
        let features = serde_json::from_slice::<'_, Vec<&str>>(&body).ok().unwrap();
        assert_eq!(features, vec![FEAT_1, FEAT_2, FEAT_3, FEAT_4]);
    }

    #[actix_rt::test]
    async fn features_integration_works() {
        let app = App::new().configure(configuration);
        let mut app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/v1/features")
            .to_request();
        let res = actix_web::test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().try_into_bytes().unwrap();
        let features = serde_json::from_slice::<'_, Vec<&str>>(&body).ok().unwrap();
        assert_eq!(features, vec![FEAT_1, FEAT_2, FEAT_3, FEAT_4]);
    }
}
