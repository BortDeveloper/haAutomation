use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod auth;
mod db;
mod http;
mod secrets;
mod sync;
mod types;
mod views;
mod yaml_io;

#[derive(Parser)]
#[command(name = "inventory", version, about = "haAutomation inventory backend")]
struct Cli {
    /// Pfad zur SQLite-Datei. Wird angelegt falls nicht vorhanden.
    #[arg(long, default_value = "inventory.db", env = "INVENTORY_DB")]
    db: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Startet den HTTP-Server.
    Serve {
        /// Listen-Adresse, z.B. 0.0.0.0:8080.
        #[arg(long, default_value = "0.0.0.0:8080", env = "INVENTORY_LISTEN")]
        listen: String,
    },
    /// Synchronisiert das Inventar aus einer Live-Quelle.
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    /// Wendet ausstehende DB-Migrationen an.
    Migrate,
}

#[derive(Subcommand)]
enum SyncSource {
    /// Home Assistant: GET /api/states gegen eine HA-Instanz.
    Ha {
        /// Basis-URL, z.B. http://homeassistant.example.local:8123.
        #[arg(long, env = "HA_URL")]
        url: String,
        /// Long-Lived Access Token. Auch ueber HA_TOKEN env setzbar.
        #[arg(long, env = "HA_TOKEN", hide_env_values = true)]
        token: String,
    },
    /// CCU / RaspberryMatic: GET /addons/xmlapi/devicelist.cgi.
    Ccu {
        /// Basis-URL der CCU, z.B. http://10.0.0.6
        #[arg(long, env = "CCU_URL")]
        url: String,
    },
    /// Philips Hue: REST-Call gegen eine oder mehrere Bridges (Config-Datei).
    Hue {
        /// YAML-Datei mit Liste [{ip, token, name?}, ...].
        #[arg(long, env = "HUE_CONFIG")]
        config: std::path::PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { listen } => {
            let conn = db::open(&cli.db)?;
            db::migrate(&conn)?;
            let auth_cfg = auth::Config::from_env();
            let server = http::bind(&listen)?;
            println!(
                "listening on {} (db: {}, auth_header: {}, bypass: {})",
                listen,
                cli.db.display(),
                auth_cfg.header_name,
                auth_cfg.bypass
            );
            http::serve(server, conn, auth_cfg)?;
        }
        Command::Sync { source } => match source {
            SyncSource::Ha { url, token } => {
                let conn = db::open(&cli.db)?;
                db::migrate(&conn)?;
                let entities = sync::ha::fetch_states(&url, &token)?;
                let devices = sync::ha::map_to_devices(&entities);
                let n = db::upsert_devices(&conn, &devices)?;
                println!(
                    "HA sync ok: {} entities, {} devices upserted",
                    entities.len(),
                    n
                );
            }
            SyncSource::Ccu { url } => {
                let conn = db::open(&cli.db)?;
                db::migrate(&conn)?;
                let ccu_devices = sync::ccu::fetch_devicelist(&url)?;
                let devices = sync::ccu::map_to_devices(&ccu_devices);
                let n = db::upsert_devices(&conn, &devices)?;
                let mut new_snaps = 0usize;
                for d in &ccu_devices {
                    if d.firmware.is_empty() {
                        continue;
                    }
                    if db::record_firmware_if_changed(&conn, "ccu", &d.address, &d.firmware)? {
                        new_snaps += 1;
                    }
                }
                println!(
                    "CCU sync ok: {} devices total, {} upserted, {} neue firmware-snapshots",
                    ccu_devices.len(),
                    n,
                    new_snaps
                );
            }
            SyncSource::Hue { config } => {
                let conn = db::open(&cli.db)?;
                db::migrate(&conn)?;
                let bridges = sync::hue::load_config(&config)?;
                let mut all_devices: Vec<sync::hue::HueDevice> = Vec::new();
                for b in &bridges {
                    let label = b.name.clone().unwrap_or_else(|| b.ip.clone());
                    let lights = sync::hue::fetch_lights(&b.ip, &b.token)?;
                    let sensors = sync::hue::fetch_sensors(&b.ip, &b.token)?;
                    println!(
                        "  bridge {}: {} lights, {} sensors",
                        label,
                        lights.len(),
                        sensors.len()
                    );
                    all_devices.extend(lights);
                    all_devices.extend(sensors);
                }
                let devices = sync::hue::map_to_devices(&all_devices);
                let n = db::upsert_devices(&conn, &devices)?;
                let mut new_snaps = 0usize;
                for d in &all_devices {
                    if let Some(fw) = &d.swversion {
                        if db::record_firmware_if_changed(&conn, "hue", &d.uniqueid, fw)? {
                            new_snaps += 1;
                        }
                    }
                }
                println!(
                    "Hue sync ok: {} bridges, {} devices upserted, {} neue firmware-snapshots",
                    bridges.len(),
                    n,
                    new_snaps
                );
            }
        },
        Command::Migrate => {
            let conn = db::open(&cli.db)?;
            db::migrate(&conn)?;
            println!("Migrationen angewandt: {}", cli.db.display());
        }
    }
    Ok(())
}
