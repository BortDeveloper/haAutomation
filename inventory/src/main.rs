use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod auth;
mod db;
mod git_publish;
mod http;
mod secrets;
mod sync;
mod types;
mod views;
mod yaml_io;

#[derive(Parser)]
#[command(name = "inventory", version, about = "haBortfeld inventory backend")]
struct Cli {
    /// Pfad zur SQLite-Datei. Wird angelegt falls nicht vorhanden.
    #[arg(long, default_value = "inventory.db", env = "INVENTORY_DB")]
    db: PathBuf,

    /// Verzeichnis fuer die YAML-Snapshots pro Source (source-of-truth fuer git).
    #[arg(long, default_value = "yaml", env = "INVENTORY_YAML_DIR")]
    yaml_dir: PathBuf,

    /// Bei Sync: yaml-Diff per `git commit && git push` veroeffentlichen.
    /// Setzt user.email/user.name im Repo voraus.
    #[arg(long, env = "INVENTORY_PUBLISH")]
    publish: bool,

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
        /// Basis-URL, z.B. http://homeassistant.local:8123.
        #[arg(long, env = "HA_URL")]
        url: String,
        /// Long-Lived Access Token. Auch ueber HA_TOKEN env setzbar.
        #[arg(long, env = "HA_TOKEN", hide_env_values = true)]
        token: String,
    },
    /// CCU / RaspberryMatic: GET /addons/xmlapi/devicelist.cgi.
    Ccu {
        /// Basis-URL der CCU, z.B. http://192.168.10.6
        #[arg(long, env = "CCU_URL")]
        url: String,
    },
    /// Philips Hue: REST-Call gegen eine oder mehrere Bridges (Config-Datei).
    Hue {
        /// YAML-Datei mit Liste [{ip, token, name?}, ...].
        #[arg(long, env = "HUE_CONFIG")]
        config: std::path::PathBuf,
    },
    /// Shelly: mDNS-Discovery + Per-Device-Fetch (Gen1+Gen2).
    Shelly {
        /// Explizite IPs (kann mehrfach angegeben werden). Wenn leer: mDNS.
        #[arg(long, value_delimiter = ',')]
        ip: Vec<String>,
        /// Sekunden fuer den mDNS-Scan. 0 = kein Scan.
        #[arg(long, default_value_t = 5)]
        discover_seconds: u64,
    },
}

fn maybe_publish(
    enabled: bool,
    yaml_dir: &std::path::Path,
    yaml_path: &std::path::Path,
    source: &str,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    let r = git_publish::commit_and_push(
        yaml_dir,
        &[yaml_path],
        &format!("auto-sync {source}"),
        true,
    )?;
    println!(
        "  git publish: committed={} pushed={}",
        r.committed, r.pushed
    );
    Ok(())
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
                let p = yaml_io::write_devices_for_source(&cli.yaml_dir, "ha", &devices)?;
                println!(
                    "HA sync ok: {} entities, {} devices upserted, yaml: {}",
                    entities.len(),
                    n,
                    p.display()
                );
                maybe_publish(cli.publish, &cli.yaml_dir, &p, "ha")?;
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
                let p = yaml_io::write_devices_for_source(&cli.yaml_dir, "ccu", &devices)?;
                println!(
                    "CCU sync ok: {} devices total, {} upserted, {} neue firmware-snapshots, yaml: {}",
                    ccu_devices.len(),
                    n,
                    new_snaps,
                    p.display()
                );
                maybe_publish(cli.publish, &cli.yaml_dir, &p, "ccu")?;
            }
            SyncSource::Shelly {
                ip,
                discover_seconds,
            } => {
                let conn = db::open(&cli.db)?;
                db::migrate(&conn)?;
                let mut ips: Vec<String> = ip.clone();
                if discover_seconds > 0 {
                    println!("mDNS-Scan {} s ...", discover_seconds);
                    let found = sync::shelly::discover(std::time::Duration::from_secs(
                        discover_seconds,
                    ))?;
                    println!("  {} Shelly(s) per mDNS gefunden", found.len());
                    ips.extend(found);
                }
                ips.sort();
                ips.dedup();
                let mut shellys: Vec<sync::shelly::ShellyDevice> = Vec::new();
                for addr in &ips {
                    match sync::shelly::fetch_info(addr) {
                        Ok(d) => shellys.push(d),
                        Err(e) => eprintln!("  WARN: {addr} -> {e:#}"),
                    }
                }
                let devices = sync::shelly::map_to_devices(&shellys);
                let n = db::upsert_devices(&conn, &devices)?;
                let mut new_snaps = 0usize;
                for d in &shellys {
                    if d.firmware.is_empty() {
                        continue;
                    }
                    if db::record_firmware_if_changed(&conn, "shelly", &d.mac, &d.firmware)? {
                        new_snaps += 1;
                    }
                }
                let p = yaml_io::write_devices_for_source(&cli.yaml_dir, "shelly", &devices)?;
                println!(
                    "Shelly sync ok: {} IPs gescannt, {} erreichbar, {} upserted, {} neue firmware-snapshots, yaml: {}",
                    ips.len(),
                    shellys.len(),
                    n,
                    new_snaps,
                    p.display()
                );
                maybe_publish(cli.publish, &cli.yaml_dir, &p, "shelly")?;
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
                let p = yaml_io::write_devices_for_source(&cli.yaml_dir, "hue", &devices)?;
                println!(
                    "Hue sync ok: {} bridges, {} devices upserted, {} neue firmware-snapshots, yaml: {}",
                    bridges.len(),
                    n,
                    new_snaps,
                    p.display()
                );
                maybe_publish(cli.publish, &cli.yaml_dir, &p, "hue")?;
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
