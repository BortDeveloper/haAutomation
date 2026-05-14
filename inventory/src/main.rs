use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod db;

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
    Serve,
    /// Synchronisiert das Inventar aus einer Live-Quelle.
    Sync,
    /// Wendet ausstehende DB-Migrationen an.
    Migrate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve => {
            println!("TODO S4: tiny_http-Server starten");
        }
        Command::Sync => {
            println!("TODO S9+: Sync gegen konfigurierte Quelle ausfuehren");
        }
        Command::Migrate => {
            let conn = db::open(&cli.db)?;
            db::migrate(&conn)?;
            println!("Migrationen angewandt: {}", cli.db.display());
        }
    }
    Ok(())
}
