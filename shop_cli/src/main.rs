use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dialoguer::console::Term;

use shop_backend::ShopBackend;

use shop_cli::{client::client_loop, employee::employee_loop};

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
    let cli = Cli::parse();
    if let Err(e) = dotenvy::dotenv_override() {
        if !e.not_found() {
            bail!(e)
        }
    }
    let backend = ShopBackend::init().await?;
    let term = Term::stdout();
    match cli.command {
        Commands::Client => client_loop(&term, backend).await?,
        Commands::Employee => employee_loop(&term, backend).await?,
    }
    Ok(())
}
