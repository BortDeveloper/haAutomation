use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "inventory", version, about = "haBortfeld inventory backend")]
struct Cli {
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
            println!("TODO S2: Migrationen anwenden");
        }
    }
    Ok(())
}
