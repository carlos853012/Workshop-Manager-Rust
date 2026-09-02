use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Crea un backup comprimido de la base de datos usando pg_dump.
/// Retorna la ruta del archivo .sql.gz generado.
pub async fn create_backup(
    pg_dump_path: &Path,
    database_url: &str,
    backups_dir: &Path,
) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(backups_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_file = backups_dir.join(format!("workshop_manager_backup_{}.sql.gz", timestamp));

    let output = Command::new(pg_dump_path)
        .arg("--dbname")
        .arg(database_url)
        .arg("--clean")
        .arg("--if-exists")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute pg_dump: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pg_dump failed: {}", stderr));
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    std::io::Write::write_all(&mut encoder, &output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to compress backup: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| anyhow::anyhow!("Failed to finish gzip compression: {}", e))?;

    fs::write(&backup_file, compressed)?;

    tracing::info!(backup_file = %backup_file.display(), "Backup created");
    Ok(backup_file)
}

/// Elimina backups antiguos manteniendo solo los `keep_count` más recientes.
pub fn prune_old_backups(backups_dir: &Path, keep_count: usize) -> anyhow::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(backups_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "gz")
                .unwrap_or(false)
        })
        .collect();

    if entries.len() <= keep_count {
        return Ok(());
    }

    entries.sort_by_key(|entry| {
        entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let to_remove = entries.len() - keep_count;
    for entry in entries.into_iter().take(to_remove) {
        fs::remove_file(entry.path())?;
        tracing::info!(file = %entry.path().display(), "Old backup removed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_old_backups() -> anyhow::Result<()> {
        let temp_dir = std::env::temp_dir().join("workshop_manager_backup_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir)?;

        for i in 0..5 {
            let file = temp_dir.join(format!("backup_{}.sql.gz", i));
            fs::write(&file, b"dummy")?;
            // Fuerza timestamps distintos esperando 10ms
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        prune_old_backups(&temp_dir, 2)?;

        let remaining: Vec<_> = fs::read_dir(&temp_dir)?.filter_map(|e| e.ok()).collect();
        assert_eq!(remaining.len(), 2);

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
