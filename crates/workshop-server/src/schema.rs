use sqlx::PgPool;

/// Ejecuta las migraciones SQLx empaquetadas en el binario.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    // Ensure barcode_prefix column exists after migrations
    ensure_barcode_prefix_column(pool).await?;

    tracing::info!("Migrations completed");
    Ok(())
}

/// Ensure the barcode_prefix column exists in workshops table
async fn ensure_barcode_prefix_column(pool: &PgPool) -> anyhow::Result<()> {
    // Check if table exists first
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_name = 'workshops'
        )",
    )
    .fetch_one(pool)
    .await?;

    tracing::info!("Workshops table exists: {}", table_exists);

    if !table_exists {
        tracing::warn!("Workshops table does not exist yet, skipping barcode_prefix check");
        return Ok(());
    }

    // Check if column exists
    let column_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'workshops' AND column_name = 'barcode_prefix'
        )",
    )
    .fetch_one(pool)
    .await?;

    tracing::info!("barcode_prefix column exists: {}", column_exists);

    if !column_exists {
        tracing::info!("Adding barcode_prefix column to workshops table");
        sqlx::query("ALTER TABLE workshops ADD COLUMN barcode_prefix VARCHAR(6)")
            .execute(pool)
            .await?;
        tracing::info!("barcode_prefix column added successfully");
    }

    Ok(())
}
