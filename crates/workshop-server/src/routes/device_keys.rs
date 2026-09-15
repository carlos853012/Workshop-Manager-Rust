use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use workshop_common::dto::ApiResponse;

use crate::{device_key, error::AppError, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_keys).post(generate_key))
        .route("/:id/revoke", post(revoke_key))
        .route("/:id/unbind", post(unbind_key))
}

async fn generate_key(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<device_key::GeneratedDeviceKey>>, AppError> {
    // Verificar límite de viewers
    if !device_key::can_create(&state.pool, state.config.max_viewers)
        .await
        .map_err(|_| AppError::Internal("Error verificando límite".to_string()))?
    {
        return Err(AppError::Forbidden(format!(
            "Límite de viewers alcanzado ({}/{}). Contacte al administrador.",
            device_key::count_active(&state.pool).await.unwrap_or(0),
            state.config.max_viewers
        )));
    }

    let key = device_key::create(&state.pool)
        .await
        .map_err(|_| AppError::Internal("No se pudo generar la clave".to_string()))?;
    Ok(Json(ApiResponse::success(device_key::GeneratedDeviceKey {
        key,
    })))
}

async fn list_keys(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<device_key::DeviceKeySummary>>>, AppError> {
    let keys = device_key::list(&state.pool)
        .await
        .map_err(|_| AppError::Internal("No se pudieron leer las claves".to_string()))?;
    Ok(Json(ApiResponse::success(keys)))
}

async fn revoke_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if !device_key::revoke(&state.pool, id)
        .await
        .map_err(|_| AppError::Internal("No se pudo revocar la clave".to_string()))?
    {
        return Err(AppError::NotFound("Clave no encontrada".to_string()));
    }
    Ok(Json(ApiResponse::success(())))
}

async fn unbind_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if !device_key::unbind(&state.pool, id)
        .await
        .map_err(|_| AppError::Internal("No se pudo desvincular la clave".to_string()))?
    {
        return Err(AppError::NotFound("Clave no encontrada".to_string()));
    }
    Ok(Json(ApiResponse::success(())))
}
