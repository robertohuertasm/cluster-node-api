use crate::{
    domain::{
        models::{Node, NodeStatus},
        repository::{node_repository::NodeFilter, NodeRepository},
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

const PATH: &str = "/v1/nodes";

pub fn service<R: NodeRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .wrap(HttpAuthentication::bearer(auth::validator))
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // GET
            .route("", web::get().to(get_all::<R>))
            .route("/{node_id}", web::get().to(get::<R>))
            // POST
            .route("", web::post().to(post::<R>))
            // PATCH
            .route("", web::patch().to(patch_status::<R>))
            // PUT
            .route("", web::put().to(put::<R>))
            // DELETE
            .route("/{node_id}", web::delete().to(delete::<R>)),
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodePatchDTO {
    pub id: uuid::Uuid,
    pub status: NodeStatus,
}

#[instrument(skip(repo))]
async fn get_all<R: NodeRepository>(
    filter: Option<web::Query<NodeFilter>>,
    repo: web::Data<R>,
) -> HttpResponse {
    let filter = filter.map(|f| f.into_inner());

    match repo.get_nodes(filter).await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[instrument(skip(repo))]
async fn get<R: NodeRepository>(node_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.get_node(&node_id).await {
        Ok(node) => HttpResponse::Ok().json(node),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[instrument(skip(repo))]
async fn post<R: NodeRepository>(node: web::Json<Node>, repo: web::Data<R>) -> HttpResponse {
    match repo.create_node(&node).await {
        Ok(node) => HttpResponse::Created().json(node),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn patch_status<R: NodeRepository>(
    node: web::Json<NodePatchDTO>,
    repo: web::Data<R>,
) -> HttpResponse {
    let old_node = repo.get_node(&node.id).await;

    match old_node {
        Ok(mut old) => {
            old.status = node.status;
            match repo.update_node(&old).await {
                Ok(node) => HttpResponse::Ok().json(node),
                Err(e) => HttpResponse::NotFound().body(format!("Something went wrong: {}", e)),
            }
        }
        Err(e) => HttpResponse::NotFound().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn put<R: NodeRepository>(node: web::Json<Node>, repo: web::Data<R>) -> HttpResponse {
    match repo.update_node(&node).await {
        Ok(node) => HttpResponse::Ok().json(node),
        Err(e) => HttpResponse::NotFound().body(format!("Something went wrong: {}", e)),
    }
}

#[instrument(skip(repo))]
async fn delete<R: NodeRepository>(node_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.delete_node(&node_id).await {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Something went wrong: {}", e)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::{models::NodeStatus, repository::node_repository::MockNodeRepository};
    use actix_http::{Request, StatusCode};
    use actix_web::{body::MessageBody, dev::ServiceResponse, App};
    use chrono::Utc;

    pub fn create_test_node(id: uuid::Uuid, name: String) -> Node {
        Node {
            id,
            name,
            cluster_id: uuid::Uuid::new_v4(),
            status: NodeStatus::PowerOn,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    fn valid_bearer() -> (&'static str, &'static str) {
        ("Authorization", "Bearer im_a_valid_user")
    }

    fn prepare_filter_repo(node: Node) -> MockNodeRepository {
        let mut repo = MockNodeRepository::default();
        repo.expect_get_nodes()
            .returning(move |filter| match filter {
                Some(filter) if node.name.contains(&filter.name) => Ok(vec![node.clone()]),
                None => Ok(vec![node.clone()]),
                _ => Ok(vec![]),
            });
        repo
    }

    #[actix_rt::test]
    async fn get_all_work_without_filter() {
        let test_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let repo = prepare_filter_repo(test_node.clone());
        let result = get_all(None, web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert!(nodes.len() == 1);
        assert_eq!(nodes[0], test_node);
    }

    #[actix_rt::test]
    async fn get_all_returns_filter_is_ok() {
        let node_name = "NODE_NAME".to_string();
        let test_node = create_test_node(uuid::Uuid::new_v4(), node_name.clone());

        let repo = prepare_filter_repo(test_node.clone());

        let result = get_all(
            Some(web::Query(NodeFilter {
                name: "NODE".to_string(),
            })),
            web::Data::new(repo),
        )
        .await;

        let body = result.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert!(nodes.len() == 1);
        assert_eq!(nodes[0], test_node);
    }

    #[actix_rt::test]
    async fn get_all_does_not_return_filter_is_not_ok() {
        let node_name = "NODE_NAME".to_string();
        let test_node = create_test_node(uuid::Uuid::new_v4(), node_name.clone());

        let repo = prepare_filter_repo(test_node.clone());

        let result = get_all(
            Some(web::Query(NodeFilter {
                name: "other".to_string(),
            })),
            web::Data::new(repo),
        )
        .await;

        let body = result.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert!(nodes.len() == 0);
    }

    async fn prepare_get_all_response(node: Node, req: Request) -> ServiceResponse {
        let repo = prepare_filter_repo(node);

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(service::<MockNodeRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn get_all_without_filter_integration_works() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(PATH)
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_all_response(node.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert_eq!(nodes, vec![node]);
    }

    #[actix_rt::test]
    async fn get_all_without_filter_integration_fails_if_no_authentication() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let req = actix_web::test::TestRequest::get().uri(PATH).to_request();
        let res = prepare_get_all_response(node, req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn get_all_filter_integration_works() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}?name=NODE", PATH))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_all_response(node.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert_eq!(nodes, vec![node]);
    }

    #[actix_rt::test]
    async fn get_all_filter_integration_fails_if_no_authentication() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}?name=NODE", PATH))
            .to_request();
        let res = prepare_get_all_response(node, req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn get_all_wrong_filter_integration_works() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}?name=other", PATH))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_all_response(node.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert_eq!(nodes, vec![]);
    }

    #[actix_rt::test]
    async fn get_all_wrong_filter_integration_fails_if_no_authentication() {
        let node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}?name=other", PATH))
            .to_request();
        let res = prepare_get_all_response(node, req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn get_works() {
        let node_id = uuid::Uuid::new_v4();
        let node_name = "NODE_NAME";

        let mut repo = MockNodeRepository::default();
        repo.expect_get_node().returning(move |id| {
            let node = create_test_node(*id, node_name.to_string());
            Ok(node)
        });

        let result = get(web::Path::from(node_id), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node.id, node_id);
        assert_eq!(node.name, node_name);
    }

    async fn prepare_get_response(node_name: String, req: Request) -> ServiceResponse {
        let mut repo = MockNodeRepository::default();
        repo.expect_get_node().returning(move |id| {
            let cluster = create_test_node(*id, node_name.clone());
            Ok(cluster)
        });

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(service::<MockNodeRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn get_integration_works() {
        let expected = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::get()
            .uri(&format!("{}/{}", PATH, expected.id))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_get_response(expected.name.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node.id, expected.id);
        assert_eq!(node.name, expected.name);
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
        let new_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let mut repo = MockNodeRepository::default();
        repo.expect_create_node()
            .returning(|node| Ok(node.to_owned()));

        let result = post(web::Json(new_node.clone()), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node, new_node);
    }

    async fn prepare_create_response(req: Request) -> ServiceResponse {
        let mut repo = MockNodeRepository::default();
        repo.expect_create_node()
            .returning(move |node| Ok(node.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(service::<MockNodeRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn create_integration_works() {
        let new_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::post()
            .uri(PATH)
            .set_json(new_node.clone())
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_create_response(req).await;
        assert_eq!(res.status(), StatusCode::CREATED);

        let body = res.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node, new_node);
    }

    #[actix_rt::test]
    async fn create_integration_fails_if_no_authentication() {
        let new_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let req = actix_web::test::TestRequest::post()
            .uri(PATH)
            .set_json(new_node)
            .to_request();
        let res = prepare_create_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn update_works() {
        let node_id = uuid::Uuid::new_v4();
        let node_name = "NODE_NAME";
        let new_node = create_test_node(node_id, node_name.to_string());

        let mut repo = MockNodeRepository::default();
        repo.expect_update_node()
            .returning(|node| Ok(node.to_owned()));

        let result = put(web::Json(new_node), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node.id, node_id);
        assert_eq!(node.name, node_name);
    }

    async fn prepare_update_response(req: Request) -> ServiceResponse {
        let mut repo = MockNodeRepository::default();
        repo.expect_update_node()
            .returning(move |node| Ok(node.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(service::<MockNodeRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn update_integration_works() {
        let new_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());

        let req = actix_web::test::TestRequest::put()
            .uri(PATH)
            .set_json(new_node.clone())
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_update_response(req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node, new_node);
    }

    #[actix_rt::test]
    async fn update_integration_fails_if_no_authentication() {
        let new_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let req = actix_web::test::TestRequest::put()
            .uri(PATH)
            .set_json(new_node)
            .to_request();
        let res = prepare_update_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn delete_works() {
        let node_id = uuid::Uuid::new_v4();

        let mut repo = MockNodeRepository::default();
        repo.expect_delete_node().returning(|id| Ok(id.to_owned()));

        let result = delete(web::Path::from(node_id), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let id = std::str::from_utf8(&body).ok().unwrap();

        assert_eq!(id, node_id.to_string());
    }

    async fn prepare_delete_response(req: Request) -> ServiceResponse {
        let mut repo = MockNodeRepository::default();
        repo.expect_delete_node().returning(|id| Ok(id.to_owned()));

        let app = App::new()
            .app_data(web::Data::new(repo))
            .configure(service::<MockNodeRepository>);

        let mut svc = actix_web::test::init_service(app).await;
        actix_web::test::call_service(&mut svc, req).await
    }

    #[actix_rt::test]
    async fn delete_integration_works() {
        let node_id = uuid::Uuid::new_v4();

        let req = actix_web::test::TestRequest::delete()
            .uri(&format!("{}/{}", PATH, node_id))
            .insert_header(valid_bearer())
            .to_request();

        let res = prepare_delete_response(req).await;
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().try_into_bytes().unwrap();
        let id = std::str::from_utf8(&body).ok().unwrap();

        assert_eq!(node_id.to_string(), id);
    }

    #[actix_rt::test]
    async fn delete_integration_fails_if_no_authentication() {
        let node_id = uuid::Uuid::new_v4();
        let req = actix_web::test::TestRequest::delete()
            .uri(&format!("{}/{}", PATH, node_id))
            .to_request();
        let res = prepare_delete_response(req).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
