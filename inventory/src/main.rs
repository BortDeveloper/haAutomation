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
#[command(name = "inventory", version, about = "home inventory backend")]
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

    /// Bestaetigung fuer `--publish`-Modus. Erforderlich, sobald `--publish`
    /// (bzw. `INVENTORY_PUBLISH`) aktiv ist. Wert ist die erwartete
    /// Remote-Beschreibung (URL/Name); dient als bewusster
    /// Operator-Konsens, damit Geraete-Inventory nicht versehentlich
    /// an ein unbeabsichtigtes Remote gepusht wird (Audit
    /// 2026-05-20 R-HIGH-3).
    #[arg(long = "confirm-publish-to", env = "INVENTORY_PUBLISH_CONFIRM")]
    confirm_publish_to: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Startet den HTTP-Server.
    Serve {
        /// Listen-Adresse, z.B. 100.x.x.x:8080 (Tailnet-IP).
        /// Default ist loopback-only (`127.0.0.1:8080`), damit der
        /// Dienst nicht versehentlich auf allen Interfaces lauscht
        /// (Audit 2026-05-20 R-HIGH-4). Fuer Tailnet-Zugriff explizit
        /// die Tailscale-IP angeben.
        #[arg(long, default_value = "127.0.0.1:8080", env = "INVENTORY_LISTEN")]
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
        /// Optional: fehlt `--config` und ist `HUE_CONFIG` nicht gesetzt,
        /// loggt der Befehl einen Hinweis und beendet mit Exit-Code 0
        /// (Hue ist eine optionale Sync-Quelle).
        #[arg(long, env = "HUE_CONFIG")]
        config: Option<std::path::PathBuf>,
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
    confirm_publish_to: Option<&str>,
    yaml_dir: &std::path::Path,
    yaml_path: &std::path::Path,
    source: &str,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    // R-HIGH-3 (Audit 2026-05-20): `INVENTORY_PUBLISH=true` darf
    // niemals stillschweigend Geraete-Inventory an ein Remote pushen.
    // Operator muss `--confirm-publish-to '<remote>'` setzen.
    let confirm = match confirm_publish_to {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            anyhow::bail!(
                "INVENTORY_PUBLISH/`--publish` ist aktiv, aber `--confirm-publish-to` \
                 (bzw. env INVENTORY_PUBLISH_CONFIRM) ist leer. \
                 Refusing to push device inventory to remote. \
                 Set --confirm-publish-to '<remote>' to acknowledge."
            );
        }
    };
    eprintln!(
        "[INVENTORY] PUBLISH mode active: committing+pushing inventory yaml for source '{}' to remote '{}'",
        source, confirm
    );
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
            // R-CRIT-2 (Audit 2026-05-20): AUTH_BYPASS=1 deaktiviert
            // jede Authentisierung. Stderr-Warnbanner ist Pflicht.
            if auth_cfg.bypass {
                eprintln!(
                    "WARNUNG: AUTH_BYPASS aktiv. Web-UI hat KEINE Authentisierung. \
                     Listen-Adresse: {} — stelle sicher, dass das nicht 0.0.0.0 ist, \
                     sonst ist die Inventory-UI auf allen Interfaces erreichbar.",
                    listen
                );
            }
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
                maybe_publish(
                    cli.publish,
                    cli.confirm_publish_to.as_deref(),
                    &cli.yaml_dir,
                    &p,
                    "ha",
                )?;
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
                maybe_publish(
                    cli.publish,
                    cli.confirm_publish_to.as_deref(),
                    &cli.yaml_dir,
                    &p,
                    "ccu",
                )?;
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
                maybe_publish(
                    cli.publish,
                    cli.confirm_publish_to.as_deref(),
                    &cli.yaml_dir,
                    &p,
                    "shelly",
                )?;
            }
            SyncSource::Hue { config } => {
                // Hue ist eine optionale Sync-Quelle: ohne Config wird die
                // Quelle still uebersprungen (kein Fehler).
                let config = match config {
                    Some(p) => p,
                    None => {
                        println!(
                            "hue: no config provided, skipping (use --config or HUE_CONFIG to enable)"
                        );
                        return Ok(());
                    }
                };
                if !config.exists() {
                    anyhow::bail!(
                        "hue config does not exist: {} (pass an existing YAML via --config or unset HUE_CONFIG to skip)",
                        config.display()
                    );
                }
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
                maybe_publish(
                    cli.publish,
                    cli.confirm_publish_to.as_deref(),
                    &cli.yaml_dir,
                    &p,
                    "hue",
                )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// `sync hue` ohne `--config` darf parsen (Hue ist optional).
    #[test]
    fn sync_hue_without_config_parses_and_config_is_none() {
        // Env-Var darf den Test nicht beeinflussen (clap liest sie sonst).
        std::env::remove_var("HUE_CONFIG");
        let cli = Cli::try_parse_from(["inventory", "sync", "hue"]).expect("parse ok");
        match cli.command {
            Command::Sync {
                source: SyncSource::Hue { config },
            } => assert!(
                config.is_none(),
                "expected None config when --config is absent"
            ),
            _ => panic!("expected Sync(Hue)"),
        }
    }

    /// `sync hue --config <pfad>` setzt den optionalen Pfad.
    #[test]
    fn sync_hue_with_config_parses_path() {
        std::env::remove_var("HUE_CONFIG");
        let cli = Cli::try_parse_from(["inventory", "sync", "hue", "--config", "/tmp/x.yml"])
            .expect("parse ok");
        match cli.command {
            Command::Sync {
                source: SyncSource::Hue { config },
            } => assert_eq!(
                config.as_deref(),
                Some(std::path::Path::new("/tmp/x.yml"))
            ),
            _ => panic!("expected Sync(Hue)"),
        }
    }

    /// `serve` ohne `--listen` muss auf 127.0.0.1:8080 defaulten
    /// (security-konformer Default, vgl. Audit 2026-05-20 R-HIGH-4).
    #[test]
    fn serve_listen_default_is_loopback() {
        std::env::remove_var("INVENTORY_LISTEN");
        let cli = Cli::try_parse_from(["inventory", "serve"]).expect("parse ok");
        match cli.command {
            Command::Serve { listen } => {
                assert_eq!(listen, "127.0.0.1:8080");
            }
            _ => panic!("expected Serve"),
        }
    }

    /// `serve --listen 100.64.0.1:8080` akzeptiert eine Tailnet-IP.
    #[test]
    fn serve_listen_accepts_explicit_address() {
        std::env::remove_var("INVENTORY_LISTEN");
        let cli = Cli::try_parse_from(["inventory", "serve", "--listen", "100.64.0.1:8080"])
            .expect("parse ok");
        match cli.command {
            Command::Serve { listen } => assert_eq!(listen, "100.64.0.1:8080"),
            _ => panic!("expected Serve"),
        }
    }

    /// `--confirm-publish-to` ist ein optionales Top-Level-Flag.
    #[test]
    fn confirm_publish_to_flag_parses() {
        std::env::remove_var("INVENTORY_PUBLISH_CONFIRM");
        std::env::remove_var("INVENTORY_PUBLISH");
        let cli = Cli::try_parse_from([
            "inventory",
            "--confirm-publish-to",
            "git@github.com:me/inventory.git",
            "migrate",
        ])
        .expect("parse ok");
        assert_eq!(
            cli.confirm_publish_to.as_deref(),
            Some("git@github.com:me/inventory.git")
        );
    }

    /// maybe_publish ohne confirm muss mit klarer Fehlermeldung ablehnen.
    #[test]
    fn maybe_publish_refuses_without_confirm() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let yaml = tmp.path().join("hue.yml");
        std::fs::write(&yaml, "[]").expect("write");
        // enabled=true, kein confirm -> Fehler
        let err = maybe_publish(true, None, tmp.path(), &yaml, "hue")
            .expect_err("must fail without --confirm-publish-to");
        let msg = format!("{err}");
        assert!(
            msg.contains("Refusing to push device inventory"),
            "msg should explain refusal, got: {msg}"
        );
    }

    /// maybe_publish mit publish=false ist ein No-op (auch ohne confirm).
    #[test]
    fn maybe_publish_noop_when_disabled() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let yaml = tmp.path().join("hue.yml");
        std::fs::write(&yaml, "[]").expect("write");
        maybe_publish(false, None, tmp.path(), &yaml, "hue").expect("noop ok");
    }
}
