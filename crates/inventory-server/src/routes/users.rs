use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use inventory_common::dto::{ApiResponse, PaginatedResponse};
use inventory_common::{User, UserRole};
use serde::Deserialize;
use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::auth;
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
}

#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default = "super::products::default_page")]
    page: i32,
    #[serde(default = "super::products::default_per_page")]
    per_page: i32,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    display_name: Option<String>,
    password: String,
    role: UserRole,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    display_name: Option<String>,
    role: Option<UserRole>,
    status: Option<String>,
}

async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<User>>>, AppError> {
    if params.page < 1 {
        return Err(AppError::Validation("page must be >= 1".to_string()));
    }
    if params.per_page < 1 || params.per_page > 100 {
        return Err(AppError::Validation(
            "per_page must be between 1 and 100".to_string(),
        ));
    }

    let offset = (params.page - 1) * params.per_page;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<User> = sqlx::query_as(
        "SELECT id, email, display_name, password_hash, role, status, created_at \
         FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let users: Vec<User> = items.into_iter().map(hide_password_hash).collect();

    let response = PaginatedResponse {
        items: users,
        total,
        page: params.page,
        per_page: params.per_page,
    };

    Ok(Json(ApiResponse::success(response)))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match user {
        Some(user) => Ok(Json(ApiResponse::success(hide_password_hash(user)))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

async fn create_user(
    State(state): State<AppState>,
    Extension(admin): Extension<AuthenticatedUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    validate_create_user_request(&req)?;

    let existing_email: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if existing_email.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let password_hash = auth::hash_password(&req.password)
        .map_err(|e| AppError::Internal(format!("Password hashing error: {}", e)))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, email, display_name, password_hash, role, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, 'active', $6)",
    )
    .bind(id)
    .bind(&req.email)
    .bind(&req.display_name)
    .bind(&password_hash)
    .bind(&req.role)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = User {
        id,
        email: req.email,
        display_name: req.display_name,
        password_hash: String::new(),
        role: req.role,
        status: "active".to_string(),
        created_at: now,
    };

    let mut new_values = serde_json::to_value(&user).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(admin.id),
        "create",
        "user",
        id,
        None,
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(user)))
}

async fn update_user(
    State(state): State<AppState>,
    Extension(admin): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let old_user: Option<User> = sqlx::query_as(
        "SELECT id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_user = old_user.ok_or(AppError::NotFound("User not found".to_string()))?;

    // No permitir que un admin se quite su propio rol de admin
    if admin.id == id {
        if let Some(ref new_role) = req.role {
            if !matches!(new_role, UserRole::Admin) {
                return Err(AppError::Forbidden);
            }
        }
    }

    let new_display_name = req.display_name.or(old_user.display_name.clone());
    let new_role = req.role.unwrap_or(old_user.role.clone());
    let new_status = req.status.unwrap_or(old_user.status.clone());

    if new_status != "active" && new_status != "inactive" {
        return Err(AppError::Validation(
            "status must be active or inactive".to_string(),
        ));
    }

    sqlx::query("UPDATE users SET display_name = $2, role = $3, status = $4 WHERE id = $1")
        .bind(id)
        .bind(&new_display_name)
        .bind(&new_role)
        .bind(&new_status)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = User {
        id,
        email: old_user.email.clone(),
        display_name: new_display_name,
        password_hash: String::new(),
        role: new_role,
        status: new_status,
        created_at: old_user.created_at,
    };

    let mut old_values = serde_json::to_value(&old_user).unwrap_or_default();
    redact_sensitive(&mut old_values);
    let mut new_values = serde_json::to_value(&user).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(admin.id),
        "update",
        "user",
        id,
        Some(old_values),
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(user)))
}

async fn delete_user(
    State(state): State<AppState>,
    Extension(admin): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if admin.id == id {
        return Err(AppError::Forbidden);
    }

    let old_user: Option<User> = sqlx::query_as(
        "SELECT id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_user = old_user.ok_or(AppError::NotFound("User not found".to_string()))?;

    sqlx::query("UPDATE users SET status = 'inactive' WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let mut old_values = serde_json::to_value(&old_user).unwrap_or_default();
    redact_sensitive(&mut old_values);

    audit::log_change(
        &state.pool,
        Some(admin.id),
        "delete",
        "user",
        id,
        Some(old_values),
        None,
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(())))
}

fn validate_create_user_request(req: &CreateUserRequest) -> Result<(), AppError> {
    if req.email.is_empty() || req.email.len() > 200 {
        return Err(AppError::Validation("Invalid email length".to_string()));
    }
    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

fn hide_password_hash(mut user: User) -> User {
    user.password_hash = String::new();
    user
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_create_user_request_valid() {
        let req = CreateUserRequest {
            email: "user@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            password: "password123".to_string(),
            role: UserRole::Seller,
        };

        assert!(validate_create_user_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_user_request_invalid_email() {
        let req = CreateUserRequest {
            email: "invalid".to_string(),
            display_name: None,
            password: "password123".to_string(),
            role: UserRole::Mechanic,
        };

        assert!(validate_create_user_request(&req).is_err());
    }

    #[test]
    fn test_validate_create_user_request_short_password() {
        let req = CreateUserRequest {
            email: "user@example.com".to_string(),
            display_name: None,
            password: "short".to_string(),
            role: UserRole::Admin,
        };

        assert!(validate_create_user_request(&req).is_err());
    }
}
