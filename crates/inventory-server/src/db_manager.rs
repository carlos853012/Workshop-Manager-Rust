use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::path::{Path, PathBuf};

use crate::crypto;

/// Gestiona el ciclo de vida de una instancia embebida de PostgreSQL.
pub struct DbManager {
    postgresql: PostgreSQL,
    database_url: Option<String>,
    database_name: String,
}

impl DbManager {
    /// Crea un nuevo gestor apuntando a los directorios de instalación y datos dentro de `data_dir`.
    pub fn new(data_dir: &Path) -> anyhow::Result<Self> {
        let installation_dir = data_dir.join("postgresql");
        let pg_data_dir = data_dir.join("pgdata");
        let password = load_or_generate_password(data_dir)?;

        let settings = Settings {
            installation_dir,
            data_dir: pg_data_dir,
            password,
            temporary: false,
            ..Settings::default()
        };

        let postgresql = PostgreSQL::new(settings);
        Ok(Self {
            postgresql,
            database_url: None,
            database_name: "workshop_manager".to_string(),
        })
    }

    /// Descarga/instala PostgreSQL si es necesario, inicia el servidor, crea la base de datos y retorna el connection string.
    pub async fn start(&mut self) -> anyhow::Result<String> {
        tracing::info!("PostgreSQL setup starting...");
        self.postgresql.setup().await
            .map_err(|e| {
                tracing::error!(error = %e, "PostgreSQL setup failed");
                e
            })?;
        tracing::info!("PostgreSQL setup complete, starting...");
        self.postgresql.start().await
            .map_err(|e| {
                tracing::error!(error = %e, "PostgreSQL start failed");
                e
            })?;
        tracing::info!("PostgreSQL started, checking database...");

        if !self.postgresql.database_exists(&self.database_name).await? {
            tracing::info!("Database '{}' does not exist, creating...", self.database_name);
            self.postgresql.create_database(&self.database_name).await?;
        }

        let database_url = self.postgresql.settings().url(&self.database_name);
        self.database_url = Some(database_url.clone());
        Ok(database_url)
    }

    /// Detiene el servidor PostgreSQL embebido.
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.postgresql.stop().await?;
        Ok(())
    }

    /// Retorna el connection string si el servidor ya fue iniciado.
    #[expect(dead_code)]
    pub fn database_url(&self) -> Option<&str> {
        self.database_url.as_deref()
    }

    /// Retorna el directorio donde se encuentran los binarios de PostgreSQL.
    pub fn binary_dir(&self) -> PathBuf {
        self.postgresql.settings().binary_dir()
    }

    /// Retorna el nombre de la base de datos gestionada.
    #[expect(dead_code)]
    pub fn database_name(&self) -> &str {
        &self.database_name
    }
}

impl Drop for DbManager {
    fn drop(&mut self) {
        let pg_ctl = self.postgresql.settings().binary_dir().join("pg_ctl");
        let data_dir = self.postgresql.settings().data_dir.clone();
        let _ = std::process::Command::new(&pg_ctl)
            .args(["stop", "-D", &data_dir.to_string_lossy(), "-m", "fast"])
            .output();
    }
}

fn load_or_generate_password(data_dir: &Path) -> anyhow::Result<String> {
    let password_path = data_dir.join(".postgres_password");

    if password_path.exists() {
        let encrypted = std::fs::read_to_string(&password_path)?;
        let password = crypto::decrypt(encrypted.trim())?;
        if password.is_empty() {
            return Err(anyhow::anyhow!("Stored PostgreSQL password is empty"));
        }
        return Ok(password);
    }

    let password = uuid::Uuid::new_v4().to_string();
    let encrypted = crypto::encrypt(&password)?;
    std::fs::write(&password_path, encrypted)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&password_path)?.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&password_path, permissions)?;
    }

    // En Windows, marcar como oculto
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("attrib")
            .arg("+H")
            .arg(&password_path)
            .output();
    }

    Ok(password)
}

/// Crea un pool de conexiones contra PostgreSQL.
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(60))
        .min_connections(1)
        .connect(database_url)
        .await?;
    Ok(pool)
}
