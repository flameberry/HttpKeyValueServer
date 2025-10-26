use axum::{Router, routing::get};
use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::env;
use tokio;

mod entities;
mod routes;

use crate::routes::{kv_store_delete_handler, kv_store_get_handler, kv_store_set_handler};

#[tokio::main]
async fn main() {
    // Necessary to make a connection to the database
    dotenv().expect(".env file not found");

    // Connecting the database before starting the HTML server
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://username:password@host/database?currentSchema=my_schema".to_string());

    let db: DatabaseConnection = Database::connect(db_url).await.unwrap();
    println!("Database connection established.");

    let app: Router<()> = Router::new()
        .route(
            "/kv/{key}",
            get(kv_store_get_handler)
                .put(kv_store_set_handler)
                .delete(kv_store_delete_handler),
        )
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Server running on http://127.0.0.1:3000");
}
