use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CERT_FILE: &str = "server.crt";
const KEY_FILE: &str = "server.key";

/// Configura el proveedor criptográfico de rustls.
/// Debe llamarse una vez antes de usar TLS.
pub fn init_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Carga un certificado y clave TLS desde disco, o genera uno autofirmado si no existen.
pub fn load_or_generate_tls_config(
    data_dir: &Path,
) -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_path = data_dir.join(CERT_FILE);
    let key_path = data_dir.join(KEY_FILE);

    if cert_path.exists() && key_path.exists() {
        let cert = std::fs::read(&cert_path)?;
        let key = std::fs::read(&key_path)?;
        return parse_tls_files(&cert, &key);
    }

    generate_self_signed_cert(data_dir)
}

fn parse_tls_files(
    mut cert_pem: &[u8],
    mut key_pem: &[u8],
) -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?;

    let keys: Vec<PrivateKeyDer<'static>> = rustls_pemfile::pkcs8_private_keys(&mut key_pem)
        .map(|result| result.map_err(|e| anyhow::anyhow!("{e}")).map(PrivateKeyDer::from))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

    let key = keys
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No private key found"))?;

    Ok((certs, key))
}

fn generate_self_signed_cert(
    data_dir: &Path,
) -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let key_pair = KeyPair::generate()
        .map_err(|e| anyhow::anyhow!("Failed to generate key pair: {}", e))?;

    let mut params = CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])
        .map_err(|e| anyhow::anyhow!("Failed to create certificate params: {}", e))?;
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, "WorkshopManager Server");

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| anyhow::anyhow!("Failed to self-sign certificate: {}", e))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    std::fs::create_dir_all(data_dir)?;
    std::fs::write(data_dir.join(CERT_FILE), &cert_pem)?;
    std::fs::write(data_dir.join(KEY_FILE), &key_pem)?;

    parse_tls_files(cert_pem.as_bytes(), key_pem.as_bytes())
}

/// Crea una configuración rustls ServerConfig a partir del certificado y clave.
pub fn create_rustls_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> anyhow::Result<Arc<rustls::ServerConfig>> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("Failed to create TLS config: {}", e))?;

    Ok(Arc::new(config))
}

/// Crea una configuración RustlsConfig para axum-server.
pub fn create_axum_rustls_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {
    let rustls_config = create_rustls_config(certs, key)?;
    Ok(axum_server::tls_rustls::RustlsConfig::from_config(rustls_config))
}

/// Retorna la ruta del certificado TLS para que el viewer pueda validarla.
#[allow(dead_code)]
pub fn cert_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CERT_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_load_self_signed_cert() -> anyhow::Result<()> {
        init_crypto_provider();

        let temp_dir = std::env::temp_dir().join("workshop_manager_tls_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        let (certs1, _key1) = generate_self_signed_cert(&temp_dir)?;
        assert!(!certs1.is_empty());

        // Segunda carga debe leer desde disco
        let (certs2, key2) = load_or_generate_tls_config(&temp_dir)?;
        assert!(!certs2.is_empty());

        // Ambas claves deben ser parseables en una config
        let _ = create_rustls_config(certs2, key2)?;

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
