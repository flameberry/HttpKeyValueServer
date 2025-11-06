use crate::{
    entities::kv_store,
    routes::{kv_store_delete_handler, kv_store_get_handler, kv_store_set_handler},
};
use axum::{Router, routing::get};
use dotenvy::dotenv;
use moka::future::Cache;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{
    env,
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};
use tokio;

mod entities;
mod routes;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    memcache: Cache<String, kv_store::Model>,
    hits: Arc<AtomicU64>,
    total_accesses: Arc<AtomicU64>,
}

const MAX_CONNECTIONS: u32 = 32;

#[tokio::main]
async fn main() {
    // Necessary to make a connection to the database
    dotenv().expect(".env file not found");

    // Connecting the database before starting the HTML server
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://username:password@host/database?currentSchema=my_schema".to_string());

    let mut connectopts = ConnectOptions::new(db_url);
    connectopts.max_connections(MAX_CONNECTIONS);

    let db: DatabaseConnection = Database::connect(connectopts).await.unwrap();
    println!("Database connection established.");

    // memcache
    let memcache: Cache<String, kv_store::Model> = Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(5 * 60))
        .build();

    let state = AppState {
        db,
        memcache,
        hits: Arc::new(AtomicU64::default()),
        total_accesses: Arc::new(AtomicU64::default()),
    };

    let app: Router<()> = Router::new()
        .route(
            "/kv/{key}",
            get(kv_store_get_handler)
                .put(kv_store_set_handler)
                .delete(kv_store_delete_handler),
        )
        .with_state(state);

    println!("Server running on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
