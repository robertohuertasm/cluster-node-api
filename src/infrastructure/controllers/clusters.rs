use crate::{
    domain::{models::Cluster, repository::ClusterRepository},
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

const PATH: &str = "/v1/clusters";

pub fn configuration<R: ClusterRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .wrap(HttpAuthentication::bearer(auth::validator))
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // GET
            .route("", web::get().to(get_all::<R>))
            .route("/{cluster_id}", web::get().to(get::<R>))
            // POST
            .route("", web::post().to(post::<R>))
            // PUT
            .route("", web::put().to(put::<R>))
            // DELETE
            .route("/{cluster_id}", web::delete().to(delete::<R>)),
    );
}

#[instrument(skip(repo))]
async fn get_all<R: ClusterRepository>(repo: web::Data<R>) -> HttpResponse {
    match repo.get_clusters().await {
        Ok(clusters) => HttpResponse::Ok().json(clusters),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[instrument(skip(repo))]
async fn get<R: ClusterRepository>(
    cluster_id: web::Path<Uuid>,
    repo: web::Data<R>,
) -> HttpResponse {
    match repo.get_cluster(&cluster_id).await {
        Ok(cluster) => HttpResponse::Ok().json(cluster),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[instrument(skip(repo))]
async fn post<R: ClusterRepository>(
    cluster: web::Json<Cluster>,
    repo: web::Data<R>,
) -> HttpResponse {
    match repo.create_cluster(&cluster).await {
        Ok(cluster) => HttpResponse::Created().json(cluster),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn put<R: ClusterRepository>(
    cluster: web::Json<Cluster>,
    repo: web::Data<R>,
) -> HttpResponse {
    match repo.update_cluster(&cluster).await {
        Ok(cluster) => HttpResponse::Ok().json(cluster),
        Err(e) => HttpResponse::NotFound().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn delete<R: ClusterRepository>(
    cluster_id: web::Path<Uuid>,
    repo: web::Data<R>,
) -> HttpResponse {
    match repo.delete_cluster(&cluster_id).await {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::repository::cluster_repository::MockClusterRepository;
    use actix_http::Request;
    use actix_web::{body::MessageBody, dev::ServiceResponse, http::StatusCode, App};
    use chrono::Utc;

    fn create_test_cluster(id: uuid::Uuid, name: String) -> Cluster {
        Cluster {
            id,
            name,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    fn valid_bearer() -> (&'static str, &'static str) {
        ("Authorization", "Bearer im_a_valid_user")
    }

    #[actix_rt::test]
    async fn get_all_works() {
        let test_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());
        let test_cluster_clone = test_cluster.clone();

        let mut repo = MockClusterRepository::default();
        repo.expect_get_clusters()
            .returning(move || Ok(vec![test_cluster_clone.clone()]));

        let res = get_all(web::Data::new(repo)).await;

        let body = res.into_body().try_into_bytes().unwrap();
        let clusters = serde_json::from_slice::<'_, Vec<Cluster>>(&body)
            .ok()
            .unwrap();

        assert!(clusters.len() == 1);
        assert_eq!(clusters[0], test_cluster);
    }

    async fn prepare_get_all_response(cluster: Cluster, req: Request) -> ServiceResponse {
        let mut repo = MockClusterRepository::default();
        repo.expect_get_clusters()
            .returning(move || Ok(vec![cluster.clone()]));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(configuration::<MockClusterRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn get_all_integration_works() {
        let cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(PATH)
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_all_response(cluster.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let clusters = serde_json::from_slice::<'_, Vec<Cluster>>(&body)
            .ok()
            .unwrap();

        assert_eq!(clusters, vec![cluster]);
    }

    #[actix_rt::test]
    async fn get_all_integration_fails_if_no_authentication() {
        let cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());
        let req = actix_web::test::TestRequest::get().uri(PATH).to_request();
        let res = prepare_get_all_response(cluster, req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn get_works() {
        let cluster_id = uuid::Uuid::new_v4();
        let cluster_name = "CLUSTER_NAME";

        let mut repo = MockClusterRepository::default();
        repo.expect_get_cluster().returning(move |id| {
            let cluster = create_test_cluster(*id, cluster_name.to_string());
            Ok(cluster)
        });

        let result = get(web::Path::from(cluster_id), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster.id, cluster_id);
        assert_eq!(cluster.name, cluster_name);
    }

    async fn prepare_get_response(cluster_name: String, req: Request) -> ServiceResponse {
        let mut repo = MockClusterRepository::default();
        repo.expect_get_cluster().returning(move |id| {
            let cluster = create_test_cluster(*id, cluster_name.clone());
            Ok(cluster)
        });

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(configuration::<MockClusterRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn get_integration_works() {
        let expected = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}/{}", PATH, expected.id))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_response(expected.name.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster.id, expected.id);
        assert_eq!(cluster.name, expected.name);
    }

    #[actix_rt::test]
    async fn get_integration_fails_if_no_authentication() {
        let cluster_id = uuid::Uuid::new_v4();
        let cluster_name = "CLUSTER_NAME";
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}/{}", PATH, cluster_id))
            .to_request();
        let res = prepare_get_response(cluster_name.to_string(), req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn create_works() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let mut repo = MockClusterRepository::default();
        repo.expect_create_cluster()
            .returning(|cluster| Ok(cluster.to_owned()));

        let result = post(web::Json(new_cluster.clone()), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster, new_cluster);
    }

    async fn prepare_create_response(req: Request) -> ServiceResponse {
        let mut repo = MockClusterRepository::default();
        repo.expect_create_cluster()
            .returning(move |cluster| Ok(cluster.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(configuration::<MockClusterRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn create_integration_works() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let req = actix_web::test::TestRequest::post()
            .uri(PATH)
            .set_json(new_cluster.clone())
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_create_response(req).await;
        assert_eq!(res.status(), StatusCode::CREATED);

        let body = res.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster, new_cluster);
    }

    #[actix_rt::test]
    async fn create_integration_fails_if_no_authentication() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());
        let req = actix_web::test::TestRequest::post()
            .uri(PATH)
            .set_json(new_cluster)
            .to_request();
        let res = prepare_create_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn update_works() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let mut repo = MockClusterRepository::default();
        repo.expect_update_cluster()
            .returning(|cluster| Ok(cluster.to_owned()));

        let result = put(web::Json(new_cluster.clone()), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster, new_cluster);
    }

    async fn prepare_update_response(req: Request) -> ServiceResponse {
        let mut repo = MockClusterRepository::default();
        repo.expect_update_cluster()
            .returning(move |cluster| Ok(cluster.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(configuration::<MockClusterRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn update_integration_works() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());

        let req = actix_web::test::TestRequest::put()
            .uri(PATH)
            .set_json(new_cluster.clone())
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_update_response(req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster, new_cluster);
    }

    #[actix_rt::test]
    async fn update_integration_fails_if_no_authentication() {
        let new_cluster = create_test_cluster(uuid::Uuid::new_v4(), "CLUSTER_NAME".to_string());
        let req = actix_web::test::TestRequest::put()
            .uri(PATH)
            .set_json(new_cluster)
            .to_request();
        let res = prepare_update_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn delete_works() {
        let cluster_id = uuid::Uuid::new_v4();

        let mut repo = MockClusterRepository::default();
        repo.expect_delete_cluster()
            .returning(|id| Ok(id.to_owned()));

        let result = delete(web::Path::from(cluster_id), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let id = std::str::from_utf8(&body).ok().unwrap();

        assert_eq!(id, cluster_id.to_string());
    }

    async fn prepare_delete_response(req: Request) -> ServiceResponse {
        let mut repo = MockClusterRepository::default();
        repo.expect_delete_cluster()
            .returning(|id| Ok(id.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(configuration::<MockClusterRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn delete_integration_works() {
        let cluster_id = uuid::Uuid::new_v4();

        let req = actix_web::test::TestRequest::delete()
            .uri(&format!("{}/{}", PATH, cluster_id))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_delete_response(req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let id = std::str::from_utf8(&body).ok().unwrap();

        assert_eq!(cluster_id.to_string(), id);
    }

    #[actix_rt::test]
    async fn delete_integration_fails_if_no_authentication() {
        let cluster_id = uuid::Uuid::new_v4();
        let req = actix_web::test::TestRequest::delete()
            .uri(&format!("{}/{}", PATH, cluster_id))
            .to_request();
        let res = prepare_delete_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
