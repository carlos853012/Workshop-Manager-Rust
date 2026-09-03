use axum::{
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use inventory_common::dto::{ApiResponse, LoginRequest, LoginResponse, RegisterRequest};
use inventory_common::UserRole;
use inventory_common::{User, Workshop};
use uuid::Uuid;

use crate::auth::{self, Claims};
use crate::error::AppError;
use crate::state::AppState;

/// Rutas públicas de autenticación: login y registro inicial.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

/// Rutas protegidas de autenticación: estado del usuario autenticado.
pub fn protected_routes() -> Router<AppState> {
    Router::new().route("/status", get(status))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    validate_email(&req.email)?;

    let user: Option<User> = sqlx::query_as(
        "SELECT id, workshop_id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE email = $1 AND status = 'active'",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = user.ok_or(AppError::Unauthorized)?;

    let valid = auth::verify_password(&user.password_hash, &req.password)
        .map_err(|e| AppError::Internal(format!("Password verification error: {}", e)))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = auth::create_token(
        user.id,
        &user.email,
        user.role.clone(),
        &state.secrets.jwt_secret,
    )
    .map_err(|e| AppError::Internal(format!("Token creation error: {}", e)))?;

    let response = LoginResponse {
        token,
        workshop: find_workshop(&state.pool, user.workshop_id).await?,
        user: hide_password_hash(user),
    };

    Ok(Json(ApiResponse::success(response)))
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    validate_workshop_field(&req.workshop_name, "workshop name", 200)?;
    validate_workshop_field(&req.workshop_address, "workshop address", 300)?;
    validate_workshop_field(&req.workshop_city, "workshop city", 120)?;
    validate_workshop_field(&req.admin_name, "admin name", 200)?;
    validate_email(&req.email)?;
    validate_password(&req.password)?;

    // Solo se permite el primer registro (admin inicial)
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if existing_count > 0 {
        return Err(AppError::Forbidden);
    }

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

    let user_id = Uuid::new_v4();
    let workshop_id = Uuid::new_v4();
    let role = UserRole::Admin;
    let now = chrono::Utc::now();
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    sqlx::query(
        "INSERT INTO workshops (id, name, address, city, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(workshop_id)
    .bind(&req.workshop_name)
    .bind(&req.workshop_address)
    .bind(&req.workshop_city)
    .bind(now)
    .execute(&mut *transaction)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    sqlx::query(
        "INSERT INTO users (id, workshop_id, email, display_name, password_hash, role, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, 'active', $7)",
    )
    .bind(user_id)
    .bind(workshop_id)
    .bind(&req.email)
    .bind(&req.admin_name)
    .bind(&password_hash)
    .bind(&role)
    .bind(now)
    .execute(&mut *transaction)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let token = auth::create_token(user_id, &req.email, role.clone(), &state.secrets.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token creation error: {}", e)))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = User {
        id: user_id,
        workshop_id,
        email: req.email,
        display_name: Some(req.admin_name),
        password_hash: String::new(),
        role,
        status: "active".to_string(),
        created_at: chrono::Utc::now(),
    };

    let workshop = Workshop {
        id: workshop_id,
        name: req.workshop_name,
        address: req.workshop_address,
        city: req.workshop_city,
        created_at: now,
        updated_at: now,
    };
    let response = LoginResponse {
        token,
        user,
        workshop: Some(workshop),
    };
    Ok(Json(ApiResponse::success(response)))
}

async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let claims = extract_claims(&headers, &state.secrets.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let user: Option<User> = sqlx::query_as(
        "SELECT id, workshop_id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = user.ok_or(AppError::Unauthorized)?;
    Ok(Json(ApiResponse::success(hide_password_hash(user))))
}

async fn find_workshop(
    pool: &sqlx::PgPool,
    workshop_id: Uuid,
) -> Result<Option<Workshop>, AppError> {
    sqlx::query_as(
        "SELECT id, name, address, city, created_at, updated_at FROM workshops WHERE id = $1",
    )
    .bind(workshop_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))
}

fn validate_workshop_field(value: &str, field: &str, max_length: usize) -> Result<(), AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > max_length {
        return Err(AppError::Validation(format!(
            "{} must be between 1 and {} characters",
            field, max_length
        )));
    }
    Ok(())
}

fn extract_claims(headers: &HeaderMap, secret: &str) -> Result<Claims, AppError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?;

    let auth_value = auth_header.to_str().map_err(|_| AppError::Unauthorized)?;

    if !auth_value.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    let token = &auth_value[7..];
    auth::validate_token(token, secret).map_err(|_| AppError::Unauthorized)
}

fn hide_password_hash(mut user: User) -> User {
    user.password_hash = String::new();
    user
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if email.is_empty() || email.len() > 200 {
        return Err(AppError::Validation("Invalid email length".to_string()));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}
