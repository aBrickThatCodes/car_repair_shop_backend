use std::process::exit;

use anyhow::{bail, Result};
use console::Term;
use dialoguer::*;
use shop_backend::*;

use crate::common::*;

pub async fn client_loop(term: &Term, mut backend: ShopBackend) -> Result<()> {
    static CLIENT_OPTIONS: [&str; 6] = [
        "Register car",
        "Register order",
        "List orders",
        "List reports",
        "Print report summary",
        "Log out",
    ];

    loop {
        let user = login_screen(term, &mut backend).await?;

        loop {
            let car = backend.get_car(user.id()).await?;

            term.clear_screen()?;

            term.write_line("Car Repair Shop Account")?;

            term.write_line(&format!("User: {}(ID: {})", user.name(), user.id()))?;
            if let Some(car) = &car {
                term.write_line(car)?;
            }

            let choice = Select::new().items(&CLIENT_OPTIONS).default(0).interact()?;
            term.clear_screen()?;

            match choice {
                0 => register_car(term, &backend, user.id(), car).await?,
                1 => register_order(term, &backend, user.id(), car).await?,
                2 => list_orders(term, &backend).await?,
                3 => list_reports(term, &backend).await?,
                4 => print_summary(term, &backend).await?,
                5 => {
                    backend.log_out().await;
                    break;
                }
                _ => unreachable!(),
            }
        }
    }
}

async fn login_screen(term: &Term, backend: &mut ShopBackend) -> Result<User> {
    loop {
        term.clear_screen()?;
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
            input(term, "Name")?
        } else {
            term.write_line("Login")?;
            String::new()
        };

        let email = input(term, "Email")?;
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
            Ok(client) => break Ok(client),
        }
    }
}

async fn register_car(
    term: &Term,
    backend: &ShopBackend,
    client_id: i32,
    car: Option<String>,
) -> Result<()> {
    match car {
        Some(_) => term.write_line("You already have registered a car")?,

        None => {
            term.write_line("Register car")?;
            let make: String = input(term, "Make")?;
            let model: String = input(term, "Model")?;
            backend.register_car(client_id, &make, &model).await?;
            term.write_line(&format!("{make} {model} registered"))?;
        }
    }

    wait_for_continue(term)?;
    Ok(())
}

async fn register_order(
    term: &Term,
    backend: &ShopBackend,
    client_id: i32,
    car: Option<String>,
) -> Result<()> {
    match car {
        Some(_) => {
            static SERVICES: [Service; 2] = [Service::Inspection, Service::Repair];

            term.write_line("Register order")?;
            let service = Select::new()
                .items(&SERVICES)
                .item("Cancel")
                .default(0)
                .interact_on(term)?;

            if service == 2 {
                return Ok(());
            }

            let service = &SERVICES[service];
            backend.register_order(client_id, service).await?;
            term.write_line(&format!(
                "Order for {} registered",
                format!("{service}").to_lowercase()
            ))?;
            wait_for_continue(term)?;
        }
        None => {
            term.write_line("You have to have a car registered to create an order")?;
            wait_for_continue(term)?;
        }
    }

    Ok(())
}

async fn list_orders(term: &Term, backend: &ShopBackend) -> Result<()> {
    {
        term.write_line("List orders")?;
        let orders = backend.get_client_orders().await?;

        if orders.is_empty() {
            term.write_line("You have no orders registered")?;
            wait_for_continue(term)?;
            return Ok(());
        }

        for order in orders {
            term.write_line(&order)?;
        }

        wait_for_continue(term)?;
        Ok(())
    }
}

async fn list_reports(term: &Term, backend: &ShopBackend) -> Result<()> {
    term.write_line("List reports")?;
    let reports = backend.get_client_reports().await?;

    if reports.is_empty() {
        term.write_line("You have no reports registered")?;
        wait_for_continue(term)?;
        return Ok(());
    }

    for report in reports {
        term.write_line(&report)?;
    }
    wait_for_continue(term)?;

    Ok(())
}

async fn print_summary(term: &Term, backend: &ShopBackend) -> Result<()> {
    let report_id = loop {
        term.write_line("Get report summary")?;
        let report_id: String = Input::new()
            .with_prompt("Report ID (or nothing to go back)")
            .default("-1".to_string())
            .interact_text_on(term)?;
        match report_id.parse::<i32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                continue;
            }
        }
    };

    if report_id == -1 {
        return Ok(());
    }

    let summary = match backend.get_report_summary(report_id).await {
        Ok(s) => s,
        Err(e) => match e.downcast_ref::<DbError>() {
            Some(DbError(s)) => {
                term.write_line(s)?;
                wait_for_continue(term)?;
                return Ok(());
            }
            None => bail!(e),
        },
    };
    term.write_line(&summary)?;
    wait_for_continue(term)?;
    Ok(())
}
