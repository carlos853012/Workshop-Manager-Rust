use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/seed", axum::routing::get(run_seed))
}

async fn run_seed(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    // Find the seed file relative to the executable
    let exe_path = std::env::current_exe().map_err(|e| AppError::Internal(e.to_string()))?;
    let seed_path = exe_path
        .parent()
        .ok_or_else(|| AppError::Internal("Cannot determine exe parent".into()))?
        .join("seeds")
        .join("seed.sql");

    if !seed_path.exists() {
        // Try relative to working directory (dev mode)
        let dev_path = std::path::PathBuf::from("crates/workshop-server/seeds/seed.sql");
        if dev_path.exists() {
            return execute_seed(&state, &dev_path).await;
        }
        return Err(AppError::NotFound(format!(
            "Seed file not found at {}",
            seed_path.display()
        )));
    }

    execute_seed(&state, &seed_path).await
}

async fn execute_seed(
    state: &AppState,
    seed_path: &std::path::Path,
) -> Result<Json<Value>, AppError> {
    let sql = std::fs::read_to_string(seed_path)
        .map_err(|e| AppError::Internal(format!("Failed to read seed file: {}", e)))?;

    // Split by semicolons and execute each statement (skip empty/whitespace-only)
    let statements: Vec<&str> = sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
        .collect();

    let mut executed = 0;
    let mut errors = Vec::new();

    for stmt in statements {
        if let Err(e) = sqlx::raw_sql(stmt).execute(&state.pool).await {
            let msg = e.to_string();
            // Ignore duplicate key errors (ON CONFLICT DO NOTHING)
            if !msg.contains("duplicate key") {
                errors.push(format!("Statement error: {}", msg));
            }
        } else {
            executed += 1;
        }
    }

    if errors.is_empty() {
        Ok(Json(json!({
            "status": "ok",
            "executed": executed,
            "message": format!("Seed completado: {} statements ejecutados", executed)
        })))
    } else {
        Ok(Json(json!({
            "status": "partial",
            "executed": executed,
            "errors": errors,
            "message": format!("Seed completado con {} warnings", errors.len())
        })))
    }
}
