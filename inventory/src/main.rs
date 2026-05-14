use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod db;
mod http;
mod types;
mod yaml_io;

#[derive(Parser)]
#[command(name = "inventory", version, about = "haBortfeld inventory backend")]
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
    Sync,
    /// Wendet ausstehende DB-Migrationen an.
    Migrate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { listen } => {
            let server = http::bind(&listen)?;
            println!("listening on {}", listen);
            http::serve(server)?;
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
