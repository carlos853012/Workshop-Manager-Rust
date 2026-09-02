// Middleware module
// TODO: Implementar en Fase 3

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn authenticate_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // TODO: Implementar verificación JWT
    let response = next.run(request).await;
    Ok(response)
}

pub async fn require_admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // TODO: Implementar verificación de rol admin
    let response = next.run(request).await;
    Ok(response)
}
