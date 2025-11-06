use std::sync::atomic::Ordering;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    entities::kv_store::{self, Entity as Kv},
};

#[derive(Deserialize, Serialize)]
pub struct SetValue {
    value: String,
}

/*
*
* 1. Checks in in-memory cache for the key value pair
* 2. If hit then returns the kv pair as a response
* 3. If miss then retrieves kv pair from the database
*
*/
pub async fn kv_store_get_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<kv_store::Model>, (StatusCode, String)> {
    // Printing stats just before executing a request
    kv_store_print_cache_stats(&state);

    // 1. Try to retrieve the kv-pair from in-memory cache
    if let Some(model) = state.memcache.get(&key).await {
        println!("HIT cache for key: {}", key);

        // Updating stats
        state.hits.fetch_add(1, Ordering::SeqCst);
        state.total_accesses.fetch_add(1, Ordering::SeqCst);
        return Ok(Json(model));
    }

    // In-Memory-Cache miss
    println!("MISS cache for key: {}", key);

    // 2. Lookup in the database
    let find_result = Kv::find()
        .filter(kv_store::Column::Key.eq(key.clone()))
        .one(&state.db)
        .await;

    match find_result {
        Ok(Some(model)) => {
            // Updating stats
            state.total_accesses.fetch_add(1, Ordering::SeqCst);
            // Case 1: Database query succeeded, and we found the item.
            // Store the retrieved key-value pair into the in-memory cache
            state.memcache.insert(key, model.clone()).await;
            Ok(Json(model))
        }
        Ok(None) => {
            // Case 2: Database query succeeded, but no item matched the key.
            Err((StatusCode::NOT_FOUND, format!("Key '{}' not found", key)))
        }
        Err(db_err) => {
            // Case 3: The database query itself failed.
            // We return a 500 error and log the details.
            eprintln!("Database error: {}", db_err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))
        }
    }
}

/*
* Creates/Updates a/the key value pair in the database and then in the in-memory cache
*/
pub async fn kv_store_set_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<SetValue>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Check if the key-value pair already exists
    let existing_model = Kv::find()
        .filter(kv_store::Column::Key.eq(key.clone()))
        .one(&state.db)
        .await;

    match existing_model {
        Ok(Some(model)) => {
            // 2a. It exists, so we update it
            let mut active_model: kv_store::ActiveModel = model.into();
            active_model.value = Set(payload.value);

            match active_model.update(&state.db).await {
                Ok(updated_model) => {
                    // 3. Insert into cache the updated key value pair
                    state.memcache.insert(key, updated_model.clone()).await;

                    // 4. Return 200 OK and the updated JSON
                    Ok((StatusCode::OK, Json(updated_model)))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update item: {}", e),
                )),
            }
        }
        Ok(None) => {
            // 2b. It doesn't exist, so create a key value pair
            let new_model = kv_store::ActiveModel {
                key: Set(key.clone()),
                value: Set(payload.value),
                id: Set(uuid::Uuid::new_v4()),
                ..Default::default()
            };

            match new_model.insert(&state.db).await {
                Ok(inserted_model) => {
                    // 3. Insert into cache the updated key value pair
                    state.memcache.insert(key, inserted_model.clone()).await;

                    // 4. Return 201 CREATED and the new JSON
                    Ok((StatusCode::CREATED, Json(inserted_model)))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create item: {}", e),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error on find: {}", e),
        )),
    }
}

/*
* Deletes the key value pair from the database and then from the in-memory cache
*/
pub async fn kv_store_delete_handler(State(state): State<AppState>, Path(key): Path<String>) -> (StatusCode, String) {
    // 1. Delete the key value pair from the database
    match Kv::delete_many()
        .filter(kv_store::Column::Key.eq(key.clone()))
        .exec(&state.db)
        .await
    {
        Ok(_) => {
            // 2. Invalidate the key from the memcache to avoid sending deleted data in a get reequest
            state.memcache.invalidate(&key).await;

            // 3. Return OK indicating which key has been deleted
            (StatusCode::OK, format!("Deleted key: '{}' successfully.", key))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error.".to_string()),
    }
}

pub fn kv_store_print_cache_stats(state: &AppState) {
    let hits = state.hits.load(Ordering::SeqCst);
    let total_accesses = state.total_accesses.load(Ordering::SeqCst);

    println!(
        "Hits: {}, Total: {}, Hit Rate: {}",
        hits,
        total_accesses,
        hits as f64 * 100.0 / total_accesses as f64
    );
}
