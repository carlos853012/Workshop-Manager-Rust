use axum::{extract::State, Json, Router};
use workshop_common::dto::ApiResponse;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct ConfigResponse {
    pub iva_rate: f64,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("", axum::routing::get(get_config))
}

async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ConfigResponse>>, AppError> {
    Ok(Json(ApiResponse::success(ConfigResponse {
        iva_rate: state.config.iva_rate,
    })))
}
