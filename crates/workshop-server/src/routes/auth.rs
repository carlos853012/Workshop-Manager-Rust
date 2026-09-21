use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use workshop_common::dto::{ApiResponse, LoginRequest, LoginResponse, RegisterRequest};
use workshop_common::UserRole;
use workshop_common::{User, Workshop};

use crate::auth;
use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{hide_password_hash, validate_email, validate_password};

/// Rutas públicas de autenticación: login, registro inicial y estado de setup.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/setup-status", get(setup_status))
}

/// Rutas protegidas de autenticación: estado del usuario autenticado y licencia.
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/license", get(license_status))
}

/// Indica si ya existe al menos un usuario en el sistema.
/// Público: no requiere autenticación.
async fn setup_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let has_users: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    Ok(Json(ApiResponse::success(
        serde_json::json!({ "has_users": has_users }),
    )))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    validate_email(&req.email)?;

    let rate_key = format!("login:{}", req.email);
    if !state
        .login_rate_limiter
        .check(&rate_key)
        .map_err(|e| AppError::Internal(format!("Rate limiter error: {}", e)))?
    {
        return Err(AppError::TooManyRequests);
    }

    let user: Option<User> = sqlx::query_as(
        "SELECT id, workshop_id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE email = $1 AND status = 'active'",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = user.ok_or_else(|| {
        let _ = state.login_rate_limiter.record_attempt(&rate_key);
        AppError::Unauthorized
    })?;

    let valid = auth::verify_password(&user.password_hash, &req.password)
        .map_err(|e| AppError::Internal(format!("Password verification error: {}", e)))?;

    if !valid {
        let _ = state.login_rate_limiter.record_attempt(&rate_key);
        return Err(AppError::Unauthorized);
    }

    let _ = state.login_rate_limiter.record_attempt(&rate_key);

    let token = auth::create_token(
        user.id,
        &user.email,
        user.role.clone(),
        user.workshop_id,
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
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
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

    let token = auth::create_token(
        user_id,
        &req.email,
        role.clone(),
        workshop_id,
        &state.secrets.jwt_secret,
    )
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
        barcode_prefix: None,
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
    Extension(auth_user): Extension<crate::middleware::AuthenticatedUser>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, workshop_id, email, display_name, password_hash, role, status, created_at \
         FROM users WHERE id = $1 AND status = 'active'",
    )
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let user = user.ok_or(AppError::Unauthorized)?;
    Ok(Json(ApiResponse::success(hide_password_hash(user))))
}

#[derive(serde::Serialize)]
pub struct LicenseInfo {
    pub is_trial: bool,
    pub tier: String,
}

async fn license_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<LicenseInfo>>, AppError> {
    let info = match &state.license {
        Some(lic) => LicenseInfo {
            is_trial: lic.is_trial(),
            tier: lic.tier.to_string(),
        },
        None => LicenseInfo {
            is_trial: true,
            tier: "Trial".to_string(),
        },
    };
    Ok(Json(ApiResponse::success(info)))
}

async fn find_workshop(
    pool: &sqlx::PgPool,
    workshop_id: Uuid,
) -> Result<Option<Workshop>, AppError> {
    sqlx::query_as(
        "SELECT id, name, address, city, barcode_prefix, created_at, updated_at FROM workshops WHERE id = $1",
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
