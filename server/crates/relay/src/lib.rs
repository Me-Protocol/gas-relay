//! This module contains the relayer axum server
use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use primitives::{
    configs::ServerConfig,
    db::{create_db_instance, create_request_status_table},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
pub mod error;
pub mod handlers;

pub struct AppState {
    // this is the database client, servering as the postgres connection pool
    pub db_client: tokio_postgres::Client,
}

/// Run the relayer server
pub async fn run_relayer_server(config: ServerConfig) -> Result<(), anyhow::Error> {
    let url = config.server_url.clone();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let db_client = create_db_instance(&config.db_url).await?;

    // create table if not exists
    create_request_status_table(&db_client).await?;

    let app_state = Arc::new(AppState { db_client });

    let app = Router::new()
        .route("/", get(|| async { "Gasless Relayer." }))
        .route("/status", get(handlers::get_request_status))
        .route("/relay", post(handlers::relay_request))
        .route("/batch-relay", post(handlers::relay_request))
        .layer(cors)
        .with_state(app_state);

    tracing::info!(url);
    axum::serve(TcpListener::bind(url).await.unwrap(), app)
        .await
        .unwrap();

    Ok(())
}
