use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Too many requests")]
    TooManyRequests,
}

/// Sanitiza errores de DB para no exponer detalles internos al cliente.
pub fn sanitize_db_error(detail: &str) -> String {
    if detail.contains("duplicate key") {
        "El recurso ya existe".to_string()
    } else if detail.contains("foreign key") {
        "Referencia inválida".to_string()
    } else if detail.contains("violates not-null") {
        "Faltan campos obligatorios".to_string()
    } else if detail.contains("invalid input syntax") {
        "Formato de dato inválido".to_string()
    } else {
        "Error interno del servidor".to_string()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, sanitize_db_error(msg))
            }
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
        };

        let body = json!({
            "success": false,
            "error": message,
        });

        (status, axum::Json(body)).into_response()
    }
}
