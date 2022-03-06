use crate::domain::{cluster::Cluster, repository::cluster_repository::ClusterRepository};
use actix_web::{
    error::PathError,
    web::{self, PathConfig},
    HttpRequest, HttpResponse,
};
use tracing::instrument;
use uuid::Uuid;
use web::ServiceConfig;

const PATH: &str = "/v1/cluster";

pub fn service<R: ClusterRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // GET
            .route("/{cluster_id}", web::get().to(get::<R>))
            // POST
            .route("/", web::post().to(post::<R>))
            // PUT
            .route("/", web::put().to(put::<R>))
            // DELETE
            .route("/{cluster_id}", web::delete().to(delete::<R>)),
    );
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

#[instrument(fields( path=?_req.path()), skip(_req))]
fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    tracing::error!(error=?err, "There was an error with the path");
    actix_web::error::ErrorBadRequest(err)
}

#[cfg(test)]
mod tests {

    use crate::domain::repository::cluster_repository::MockClusterRepository;

    use super::*;
    use actix_web::body::MessageBody;
    use chrono::Utc;

    pub fn create_test_cluster(id: uuid::Uuid, name: String) -> Cluster {
        Cluster {
            id,
            name,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
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

    #[actix_rt::test]
    async fn create_works() {
        let cluster_id = uuid::Uuid::new_v4();
        let cluster_name = "CLUSTER_NAME";
        let new_cluster = create_test_cluster(cluster_id, cluster_name.to_string());

        let mut repo = MockClusterRepository::default();
        repo.expect_create_cluster()
            .returning(|cluster| Ok(cluster.to_owned()));

        let result = post(web::Json(new_cluster), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster.id, cluster_id);
        assert_eq!(cluster.name, cluster_name);
    }

    #[actix_rt::test]
    async fn update_works() {
        let cluster_id = uuid::Uuid::new_v4();
        let cluster_name = "CLUSTER_NAME";
        let new_cluster = create_test_cluster(cluster_id, cluster_name.to_string());

        let mut repo = MockClusterRepository::default();
        repo.expect_update_cluster()
            .returning(|cluster| Ok(cluster.to_owned()));

        let result = put(web::Json(new_cluster), web::Data::new(repo)).await;

        let body = result.into_body().try_into_bytes().unwrap();
        let cluster = serde_json::from_slice::<'_, Cluster>(&body).ok().unwrap();

        assert_eq!(cluster.id, cluster_id);
        assert_eq!(cluster.name, cluster_name);
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
}
