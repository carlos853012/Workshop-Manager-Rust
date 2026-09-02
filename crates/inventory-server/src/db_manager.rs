use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::path::Path;

/// Gestiona el ciclo de vida de una instancia embebida de PostgreSQL.
pub struct DbManager {
    postgresql: PostgreSQL,
}

impl DbManager {
    /// Crea un nuevo gestor apuntando a los directorios de instalación y datos dentro de `data_dir`.
    pub fn new(data_dir: &Path) -> anyhow::Result<Self> {
        let installation_dir = data_dir.join("postgresql");
        let pg_data_dir = data_dir.join("pgdata");

        let settings = Settings {
            installation_dir,
            data_dir: pg_data_dir,
            temporary: false,
            ..Settings::default()
        };

        let postgresql = PostgreSQL::new(settings);
        Ok(Self { postgresql })
    }

    /// Descarga/instala PostgreSQL si es necesario, inicia el servidor, crea la base de datos y retorna el connection string.
    pub async fn start(&mut self) -> anyhow::Result<String> {
        self.postgresql.setup().await?;
        self.postgresql.start().await?;

        let database_name = "workshop_manager";
        if !self.postgresql.database_exists(database_name).await? {
            self.postgresql.create_database(database_name).await?;
        }

        let database_url = self.postgresql.settings().url(database_name);
        Ok(database_url)
    }

    /// Detiene el servidor PostgreSQL embebido.
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.postgresql.stop().await?;
        Ok(())
    }
}

/// Crea un pool de conexiones contra PostgreSQL.
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}
