use std::process::exit;

use shop_backend::*;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::*;

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
    term.clear_screen()?;
    match cli.command {
        Commands::Client => client_loop(&term, backend).await?,
        Commands::Employee => todo!(),
    }
    Ok(())
}

fn wait_for_continue(term: &Term) -> Result<()> {
    term.write_line("Press any key to continue")?;
    term.read_key()?;
    Ok(())
}

async fn client_loop(term: &Term, mut backend: ShopBackend) -> Result<()> {
    static CLIENT_OPTIONS: [&str; 6] = [
        "Register car",
        "Create order",
        "List orders",
        "List reports",
        "Print report summary",
        "Log out",
    ];

    loop {
        let user = loop {
            term.write_line("Car Repair Shop Client Interface")?;

            let choice = Select::new()
                .items(&["Login", "Register", "Exit"][..])
                .default(0)
                .interact_on(term)?;

            if choice == 2 {
                exit(0);
            }

            term.clear_screen()?;

            let name = if choice == 1 {
                term.write_line("Register")?;
                Input::new().with_prompt("Name").interact_text()?
            } else {
                term.write_line("Login")?;
                String::new()
            };

            let email = Input::new().with_prompt("Email").interact_text_on(term)?;
            let password = Password::new().with_prompt("Password").interact_on(term)?;

            let user = match choice {
                0 => backend.client_login(&email, &password).await,
                1 => backend.register_client(&name, &email, &password).await,
                _ => unreachable!(),
            };

            term.clear_screen()?;
            match user {
                Err(e) => {
                    term.write_line(&format!("{e}"))?;
                    wait_for_continue(term)?;
                    continue;
                }
                Ok(client) => break client,
            }
        };

        let User::Client { id, email, name } = user else {
            unreachable!()
        };

        loop {
            let car = backend.get_car(id).await?;

            term.clear_screen()?;

            term.write_line("Car Repair Shop Account")?;

            term.write_line(&format!("User: {name}(ID: {id} email: {email})"))?;
            if let Some(car) = &car {
                term.write_line(car)?;
            }

            let choice = Select::new().items(&CLIENT_OPTIONS).default(0).interact()?;
            term.clear_screen()?;

            match choice {
                0 => {
                    match car {
                        Some(_) => term.write_line("You already have registered a car")?,

                        None => {
                            let make: String =
                                Input::new().with_prompt("Make").interact_text_on(term)?;
                            let model: String =
                                Input::new().with_prompt("Model").interact_text_on(term)?;
                            backend.register_car(id, &make, &model).await?;
                            term.write_line(&format!("{make} {model} registered"))?;
                        }
                    }

                    wait_for_continue(term)?
                }
                1 => match car {
                    Some(_) => {
                        static SERVICES: [Service; 2] = [Service::Inspection, Service::Repair];

                        let service = Select::new()
                            .items(&SERVICES)
                            .item("Cancel")
                            .default(0)
                            .interact_on(term)?;

                        if service == 2 {
                            continue;
                        }

                        let service = &SERVICES[service];
                        backend.register_order(id, service).await?;
                        let service = format!("{service}").to_lowercase();
                        term.write_line(&format!("Order for {service} registered"))?;
                        wait_for_continue(term)?;
                    }
                    None => {
                        term.write_line("You have to have a car registered to create an order")?;
                        wait_for_continue(term)?;
                    }
                },
                2 => {
                    term.write_line("List orders")?;
                    let orders = backend.get_client_orders().await?;

                    if orders.is_empty() {
                        term.write_line("You have no orders registered")?;
                        wait_for_continue(term)?;
                        continue;
                    }

                    for order in orders {
                        term.write_line(&order)?;
                    }

                    wait_for_continue(term)?;
                }
                3 => {
                    term.write_line("List reports")?;
                    let reports = backend.get_client_reports().await?;

                    if reports.is_empty() {
                        term.write_line("You have no reports registered")?;
                        wait_for_continue(term)?;
                        continue;
                    }

                    for report in reports {
                        term.write_line(&report)?;
                    }
                    wait_for_continue(term)?;
                }
                4 => loop {
                    term.write_line("Get report summary")?;
                    let report_id: String = Input::new()
                        .with_prompt("Report ID (or nothing to go back)")
                        .default("-1".to_string())
                        .interact_text_on(term)?;
                    let report_id = match report_id.parse::<i32>() {
                        Ok(i) => i,
                        Err(e) => {
                            term.write_line(&format!("{e}"))?;
                            wait_for_continue(term)?;
                            continue;
                        }
                    };

                    if report_id == -1 {
                        break;
                    }

                    let summary = match backend.get_report_summary(report_id).await {
                        Ok(s) => s,
                        Err(e) => match e.downcast_ref::<DbError>() {
                            Some(DbError(s)) => {
                                term.write_line(s)?;
                                continue;
                            }
                            None => bail!(e),
                        },
                    };
                    term.write_line(&summary)?;
                    wait_for_continue(term)?;
                },
                5 => break,
                _ => unreachable!(),
            }
        }
    }
}
