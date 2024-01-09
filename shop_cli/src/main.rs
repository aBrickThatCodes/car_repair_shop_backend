use shop_backend::*;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Client interface
    Client,
    /// Employee interface
    Employee,
}

#[async_std::main]
async fn main() -> Result<()> {
    Cli::parse();
    if let Err(e) = dotenvy::dotenv_override() {
        if !e.not_found() {
            bail!(e)
        }
    }
    ShopBackend::init().await?;
    Ok(())
}
