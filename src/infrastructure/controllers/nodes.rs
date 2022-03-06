use crate::domain::{
    node::{Node, NodeStatus},
    repository::NodeRepository,
};
use actix_web::{
    web::{self, PathConfig},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use web::ServiceConfig;

use super::path_config_handler;

const PATH: &str = "/v1/node";

pub fn service<R: NodeRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
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
async fn get_all<R: NodeRepository>(repo: web::Data<R>) -> HttpResponse {
    match repo.get_nodes().await {
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
    use crate::domain::{node::NodeStatus, repository::node_repository::MockNodeRepository};
    use actix_web::body::MessageBody;
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

    #[actix_rt::test]
    async fn get_all_works() {
        let test_node = create_test_node(uuid::Uuid::new_v4(), "NODE_NAME".to_string());
        let test_node_clone = test_node.clone();

        let mut repo = MockNodeRepository::default();
        repo.expect_get_nodes()
            .returning(move || Ok(vec![test_node_clone.clone()]));

        let result = get_all(web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let nodes = serde_json::from_slice::<'_, Vec<Node>>(&body).ok().unwrap();

        assert!(nodes.len() == 1);
        assert_eq!(nodes[0], test_node);
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

    #[actix_rt::test]
    async fn create_works() {
        let node_id = uuid::Uuid::new_v4();
        let node_name = "NODE_NAME";
        let new_node = create_test_node(node_id, node_name.to_string());

        let mut repo = MockNodeRepository::default();
        repo.expect_create_node()
            .returning(|node| Ok(node.to_owned()));

        let result = post(web::Json(new_node), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let node = serde_json::from_slice::<'_, Node>(&body).ok().unwrap();

        assert_eq!(node.id, node_id);
        assert_eq!(node.name, node_name);
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
}
