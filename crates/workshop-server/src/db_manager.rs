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

        // `Settings::default()` crea dos tempdirs con `.keep()` (pwfile y data)
        // que quedarían huérfanos si no los limpiamos. Los capturamos antes de
        // sobreescribirlos y los borramos solo si están dentro del temp del SO.
        let mut settings = Settings::default();
        let stray_tmp_dirs = [
            settings.password_file.parent().map(Path::to_path_buf),
            Some(settings.data_dir.clone()),
        ];

        settings.installation_dir = installation_dir;
        settings.data_dir = pg_data_dir;
        settings.password = password;
        // Ruta controlada para el `--pwfile` de initdb; se borra tras `setup()`.
        settings.password_file = data_dir.join(".pgpass_init");

        // CRÍTICO: `Settings::default()` marca `temporary = true`, y el `Drop`
        // de `PostgreSQL` hace `remove_dir_all(data_dir)`. Si el proceso termina
        // de forma normal (p. ej. `start()` falla y `main` retorna `Err`), eso
        // borraría TODO el `pgdata` y perdería la base de datos. Lo desactivamos.
        settings.temporary = false;

        for dir in stray_tmp_dirs.into_iter().flatten() {
            if dir.starts_with(std::env::temp_dir()) {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }

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
        let setup_result = self.postgresql.setup().await;

        // initdb usa `password_file` como `--pwfile` (contraseña en claro).
        // Lo borramos siempre (incluso si setup falla) para no dejarla en disco.
        let password_file = self.postgresql.settings().password_file.clone();
        if let Err(e) = std::fs::remove_file(&password_file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "No se pudo borrar el archivo temporal de contraseña ({}): {}",
                    password_file.display(),
                    e
                );
            }
        }

        setup_result.map_err(|e| {
            tracing::error!(error = %e, "PostgreSQL setup failed");
            e
        })?;
        // Antes de arrancar, limpiar una instancia previa mal cerrada (huérfana)
        // o un `postmaster.pid` obsoleto, para que `pg_ctl start` no falle.
        self.cleanup_stale_instance();
        tracing::info!("PostgreSQL setup complete, starting...");
        self.postgresql.start().await.map_err(|e| {
            tracing::error!(error = %e, "PostgreSQL start failed");
            e
        })?;
        tracing::info!("PostgreSQL started, checking database...");

        if !self.postgresql.database_exists(&self.database_name).await? {
            tracing::info!(
                "Database '{}' does not exist, creating...",
                self.database_name
            );
            self.postgresql.create_database(&self.database_name).await?;
        }

        let database_url = self.postgresql.settings().url(&self.database_name);
        self.database_url = Some(database_url.clone());
        Ok(database_url)
    }

    /// Limpia una instancia previa mal cerrada antes de arrancar.
    ///
    /// Si existe `pgdata/postmaster.pid`:
    /// - Con un postmaster vivo (huérfano, p. ej. tras matar el server), lo
    ///   detiene con `pg_ctl stop`.
    /// - Si el pid file es obsoleto (proceso muerto), lo elimina.
    ///
    /// Esto evita que `pg_ctl start` falle por una instancia colgada.
    fn cleanup_stale_instance(&self) {
        let data_dir = self.postgresql.settings().data_dir.clone();
        let pid_file = data_dir.join("postmaster.pid");
        let pg_ctl = self.postgresql.settings().binary_dir().join("pg_ctl");

        let pid_file_exists = pid_file.exists();
        if !pid_file_exists {
            return;
        }

        let data_arg = data_dir.to_string_lossy().to_string();
        let status_code = run_pg_ctl(&pg_ctl, &["status", "-D", &data_arg]);

        match decide_stale_action(pid_file_exists, status_code) {
            StaleAction::None => {}
            StaleAction::StopOrphan => {
                tracing::warn!(
                    "PostgreSQL embebido ya estaba en ejecución (huérfano) — deteniéndolo..."
                );
                let _ = run_pg_ctl(&pg_ctl, &["stop", "-D", &data_arg, "-m", "fast", "-w"]);
                tracing::warn!("Instancia previa de PostgreSQL detenida");
            }
            StaleAction::RemovePidFile => {
                tracing::warn!("postmaster.pid obsoleto detectado — eliminándolo");
                if let Err(e) = std::fs::remove_file(&pid_file) {
                    tracing::warn!("No se pudo eliminar {}: {}", pid_file.display(), e);
                }
            }
        }
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
        let mut cmd = std::process::Command::new(&pg_ctl);
        cmd.args(["stop", "-D", &data_dir.to_string_lossy(), "-m", "fast"]);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let _ = cmd.output();
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

/// Acción a tomar sobre una instancia previa de PostgreSQL.
#[derive(Debug, PartialEq)]
enum StaleAction {
    /// No hay nada que limpiar.
    None,
    /// Hay un postmaster vivo (huérfano): detenerlo.
    StopOrphan,
    /// El `postmaster.pid` es obsoleto (proceso muerto): eliminarlo.
    RemovePidFile,
}

/// Decide qué hacer con un `postmaster.pid` existente según el resultado de
/// `pg_ctl status` (`0` = servidor en ejecución; otro/`None` = sin servidor).
fn decide_stale_action(pid_file_exists: bool, status_code: Option<i32>) -> StaleAction {
    if !pid_file_exists {
        return StaleAction::None;
    }
    if status_code == Some(0) {
        StaleAction::StopOrphan
    } else {
        StaleAction::RemovePidFile
    }
}

/// Ejecuta `pg_ctl` con los argumentos dados y retorna el código de salida.
fn run_pg_ctl(program: &Path, args: &[&str]) -> Option<i32> {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.output().ok().and_then(|o| o.status.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_stale_action() {
        assert_eq!(decide_stale_action(false, None), StaleAction::None);
        assert_eq!(decide_stale_action(false, Some(0)), StaleAction::None);
        assert_eq!(decide_stale_action(true, Some(0)), StaleAction::StopOrphan);
        assert_eq!(
            decide_stale_action(true, Some(3)),
            StaleAction::RemovePidFile
        );
        assert_eq!(decide_stale_action(true, None), StaleAction::RemovePidFile);
    }
}
