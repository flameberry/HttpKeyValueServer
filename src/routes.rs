use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::entities::kv_store::{self, Entity as Kv};

#[derive(Deserialize, Serialize)]
pub struct SetValue {
    value: String,
}

pub async fn kv_store_get_handler(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(key): Path<String>,
) -> Result<Json<kv_store::Model>, (StatusCode, String)> {
    let find_result = Kv::find().filter(kv_store::Column::Key.eq(key.clone())).one(&db).await;

    match find_result {
        Ok(Some(model)) => {
            // Case 1: Database query succeeded, and we found the item.
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

// This is a PUT handler, so it should UPDATE an existing item or CREATE a new one
pub async fn kv_store_set_handler(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(key): Path<String>,
    Json(payload): Json<SetValue>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Check if the key-value pair already exists
    let existing_model = Kv::find().filter(kv_store::Column::Key.eq(key.clone())).one(&db).await;

    match existing_model {
        Ok(Some(model)) => {
            // It exists, so we update it
            let mut active_model: kv_store::ActiveModel = model.into();
            active_model.value = Set(payload.value);

            match active_model.update(&db).await {
                Ok(updated_model) => {
                    // Return 200 OK and the updated JSON
                    Ok((StatusCode::OK, Json(updated_model)))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update item: {}", e),
                )),
            }
        }
        Ok(None) => {
            let new_model = kv_store::ActiveModel {
                key: Set(key),
                value: Set(payload.value),
                id: Set(uuid::Uuid::new_v4()),
                ..Default::default()
            };

            match new_model.insert(&db).await {
                Ok(inserted_model) => {
                    // Return 201 CREATED and the new JSON
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

pub async fn kv_store_delete_handler(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(key): Path<String>,
) -> (StatusCode, String) {
    match Kv::delete_many()
        .filter(kv_store::Column::Key.eq(key.clone()))
        .exec(&db)
        .await
    {
        Ok(_) => (StatusCode::OK, format!("Deleted key: '{}' successfully.", key)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error.".to_string()),
    }
}
