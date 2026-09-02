use sqlx::PgPool;

/// Ejecuta las migraciones SQLx empaquetadas en el binario.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    tracing::info!("Migrations completed");
    Ok(())
}
