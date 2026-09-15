use clap::{Parser, Subcommand};
use workshop_common::features::{License, LicenseTier};
use workshop_common::license as lic;

#[derive(Parser)]
#[command(name = "license-tool")]
#[command(about = "Herramienta de generación de licencias para WorkshopManager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Genera un par de claves Ed25519 (vendor)
    GenerateKeypair {
        /// Ruta para guardar la clave secreta
        #[arg(long, default_value = "secret_key.bin")]
        secret: String,
        /// Ruta para guardar la clave pública (para embeber en el server)
        #[arg(long, default_value = "public_key.bin")]
        public: String,
    },

    /// Genera una licencia firmada
    Generate {
        /// Clave de licencia legible
        #[arg(long)]
        key: String,
        /// Hardware hash del PC del cliente
        #[arg(long)]
        hw: String,
        /// Tier de licencia
        #[arg(long, default_value = "base")]
        tier: LicenseTierArg,
        /// Máximo de viewers simultáneos
        #[arg(long)]
        max_viewers: Option<u32>,
        /// Máximo de migraciones
        #[arg(long)]
        max_transfers: Option<u32>,
        /// Archivo con la clave secreta del vendor
        #[arg(long, default_value = "secret_key.bin")]
        secret_key: String,
        /// Archivo de salida
        #[arg(long, default_value = "license.dat")]
        output: String,
    },

    /// Verifica una licencia
    Verify {
        /// Archivo de licencia
        #[arg(long, default_value = "license.dat")]
        input: String,
        /// Archivo con la clave pública del vendor
        #[arg(long, default_value = "public_key.bin")]
        public_key: String,
    },

    /// Migra una licencia a nuevo hardware
    Migrate {
        /// Clave de licencia
        #[arg(long)]
        key: String,
        /// Nuevo hardware hash
        #[arg(long)]
        new_hw: String,
        /// Archivo con la clave secreta del vendor
        #[arg(long, default_value = "secret_key.bin")]
        secret_key: String,
        /// Archivo de salida
        #[arg(long, default_value = "license.dat")]
        output: String,
    },

    /// Muestra el hardware hash de esta máquina
    HardwareId,
}

#[derive(clap::ValueEnum, Clone)]
enum LicenseTierArg {
    Trial,
    Base,
    Reports,
    Advanced,
    Api,
}

impl From<LicenseTierArg> for LicenseTier {
    fn from(arg: LicenseTierArg) -> Self {
        match arg {
            LicenseTierArg::Trial => LicenseTier::Trial,
            LicenseTierArg::Base => LicenseTier::Base,
            LicenseTierArg::Reports => LicenseTier::Reports,
            LicenseTierArg::Advanced => LicenseTier::Advanced,
            LicenseTierArg::Api => LicenseTier::Api,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeypair { secret, public } => {
            let (sk, pk) = lic::generate_keypair();
            std::fs::write(&secret, &sk).expect("Error guardando clave secreta");
            std::fs::write(&public, &pk).expect("Error guardando clave pública");
            println!("Claves generadas:");
            println!("  Secreta: {secret}");
            println!("  Pública: {public}");
            println!();
            println!("Pública (hex para embeber en el server):");
            println!("  {:?}", pk);
        }

        Commands::Generate {
            key,
            hw,
            tier,
            max_viewers,
            max_transfers,
            secret_key,
            output,
        } => {
            let tier: LicenseTier = tier.into();
            let license = License {
                license_key: key,
                tier: tier.clone(),
                hardware_hash: hw,
                max_viewers: max_viewers.unwrap_or(tier.max_viewers()),
                max_transfers: max_transfers.unwrap_or(tier.max_transfers()),
                transfer_count: 0,
                activated_at: chrono::Utc::now(),
            };

            let sk = std::fs::read(&secret_key).expect("Error leyendo clave secreta");
            let signed = lic::sign_license(&license, &sk).expect("Error firmando licencia");
            std::fs::write(&output, &signed).expect("Error guardando licencia");

            println!("Licencia generada:");
            println!("  Key: {}", license.license_key);
            println!("  Tier: {:?}", license.tier);
            println!("  Viewers: {}", license.max_viewers);
            println!("  Migraciones: {}", license.max_transfers);
            println!("  Hardware: {}", &license.hardware_hash[..16]);
            println!("  Archivo: {output}");
        }

        Commands::Verify { input, public_key } => {
            let data = std::fs::read(&input).expect("Error leyendo licencia");
            let pk = std::fs::read(&public_key).expect("Error leyendo clave pública");

            match lic::verify_license(&data, &pk) {
                Ok(license) => {
                    println!("Licencia válida:");
                    println!("  Key: {}", license.license_key);
                    println!("  Tier: {:?}", license.tier);
                    println!("  Viewers: {}", license.max_viewers);
                    println!(
                        "  Migraciones: {}/{}",
                        license.transfer_count, license.max_transfers
                    );
                    println!("  Hardware: {}", &license.hardware_hash[..16]);
                    println!("  Activada: {}", license.activated_at);
                }
                Err(e) => {
                    eprintln!("Licencia inválida: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Migrate {
            key,
            new_hw,
            secret_key,
            output,
        } => {
            let sk = std::fs::read(&secret_key).expect("Error leyendo clave secreta");

            // Leer licencia existente para obtener tier y transfer_count
            let license_data = if std::path::Path::new(&output).exists() {
                let pk_path = output.replace("license.dat", "public_key.bin");
                if std::path::Path::new(&pk_path).exists() {
                    let pk = std::fs::read(&pk_path).expect("Error leyendo clave pública");
                    let data = std::fs::read(&output).expect("Error leyendo licencia");
                    lic::verify_license(&data, &pk).ok()
                } else {
                    None
                }
            } else {
                None
            };

            let (tier, transfer_count) = match license_data {
                Some(lic) => (lic.tier, lic.transfer_count),
                None => {
                    eprintln!("No se encontró licencia anterior. Usando tier Base por defecto.");
                    (LicenseTier::Base, 0)
                }
            };

            let license = License {
                license_key: key,
                tier: tier.clone(),
                hardware_hash: new_hw,
                max_viewers: tier.max_viewers(),
                max_transfers: tier.max_transfers(),
                transfer_count: transfer_count + 1,
                activated_at: chrono::Utc::now(),
            };

            let signed = lic::sign_license(&license, &sk).expect("Error firmando licencia");
            std::fs::write(&output, &signed).expect("Error guardando licencia");

            println!("Licencia migrada:");
            println!("  Key: {}", license.license_key);
            println!("  Tier: {:?}", license.tier);
            println!("  Nueva hardware: {}", &license.hardware_hash[..16]);
            println!(
                "  Migración: {}/{}",
                license.transfer_count, license.max_transfers
            );
            println!("  Archivo: {output}");
        }

        Commands::HardwareId => match workshop_common::hardware::get_hardware_id() {
            Ok(hash) => {
                println!("Hardware ID: {hash}");
                println!("Código de activación: {}", &hash[..16]);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
    }
}
