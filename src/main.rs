mod domain;
pub mod infrastructure;

use crate::infrastructure::db::{PostgresClusterRepository, PostgresNodeRepository};
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use infrastructure::controllers;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init env vars
    dotenv::dotenv().ok();
    // init tracing subscriber
    let tracing = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_env_filter(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }

    let conn_str = std::env::var("DATABASE_URL").expect("No DATABASE_URL env var found");
    let pool = sqlx::PgPool::connect(&conn_str)
        .await
        .expect("Can't connect to database");

    // instantiate repos
    // pool uses arc internally so it can be cloned without any impact
    let cluster_repo = web::Data::new(PostgresClusterRepository::new(pool.clone()));
    let node_repo = web::Data::new(PostgresNodeRepository::new(pool));

    // building address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);
    // building shared state
    tracing::debug!("Starting our server at {}", address);

    // starting the server
    HttpServer::new(move || {
        let cors = Cors::default().allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"]);
        App::new()
            .wrap(middleware::NormalizePath::trim())
            .wrap(cors)
            .app_data(cluster_repo.clone())
            .app_data(node_repo.clone())
            .configure(controllers::clusters::service::<PostgresClusterRepository>)
            .configure(controllers::nodes::service::<PostgresNodeRepository>)
            .configure(controllers::health::service)
            .configure(controllers::features::service)
    })
    .bind(&address)
    .unwrap_or_else(|err| {
        panic!(
            "🔥🔥🔥 Couldn't start the server in port {}: {:?}",
            port, err
        )
    })
    .run()
    .await
}
