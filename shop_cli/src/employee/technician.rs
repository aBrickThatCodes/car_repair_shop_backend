use anyhow::{bail, Result};
use dialoguer::{console::Term, *};
use shop_backend::*;

use crate::common::*;

pub async fn technician_loop(term: &Term, backend: &mut ShopBackend, user: &User) -> Result<()> {
    static TECHNICIAN_OPTIONS: [&str; 5] = [
        "Register car",
        "Register order",
        "List finished reports",
        "Create report",
        "Log out",
    ];

    loop {
        term.clear_screen()?;
        term.write_line("Car Repair Shop Account")?;

        term.write_line(&format!(
            "{}: {}(ID: {})",
            user.user_type(),
            user.name(),
            user.id()
        ))?;

        let choice = Select::new()
            .items(&TECHNICIAN_OPTIONS)
            .default(0)
            .interact()?;
        term.clear_screen()?;

        match choice {
            0 => register_car(term, backend).await?,
            1 => register_order(term, backend).await?,
            2 => list_finished_orders(term, backend).await?,
            3 => register_report(term, backend).await?,
            4 => {
                backend.log_out().await;
                break Ok(());
            }
            _ => unreachable!(),
        }
    }
}

async fn register_car(term: &Term, backend: &ShopBackend) -> Result<()> {
    let client_id = loop {
        term.write_line("Register car")?;
        let client_id: String = Input::new()
            .with_prompt("Client ID (or nothing to go back)")
            .default("-1".to_string())
            .interact_text_on(term)?;
        match client_id.parse::<i32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                term.clear_screen()?;
                continue;
            }
        }
    };

    if client_id == -1 {
        return Ok(());
    }

    let make: String = input(term, "Make")?;
    let model: String = input(term, "Model")?;

    match backend.register_car(client_id, &make, &model).await {
        Ok(_) => term.write_line(&format!("{make} {model} registered to client {client_id}"))?,
        Err(e) => match e.downcast_ref::<DbError>() {
            Some(DbError(s)) => {
                term.write_line(s)?;
                wait_for_continue(term)?;
                term.clear_screen()?;
            }
            None => bail!(e),
        },
    }
    Ok(())
}

async fn register_order(term: &Term, backend: &ShopBackend) -> Result<()> {
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

    let client_id = loop {
        term.write_line("Register order")?;
        let client_id: String = Input::new()
            .with_prompt("Client ID (or nothing to go back)")
            .default("-1".to_string())
            .interact_text_on(term)?;
        match client_id.parse::<i32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                term.clear_screen()?;
                continue;
            }
        }
    };

    if client_id == -1 {
        return Ok(());
    }

    let service = &SERVICES[service];
    match backend.register_order(client_id, service).await {
        Ok(_) => {
            term.write_line(&format!(
                "Order for {} for client {client_id} registered",
                format!("{service}").to_lowercase()
            ))?;
            wait_for_continue(term)?;
        }
        Err(e) => term.write_line(&format!("{e}"))?,
    }

    Ok(())
}

async fn list_finished_orders(term: &Term, backend: &ShopBackend) -> Result<()> {
    term.write_line("List reports")?;
    let orders = backend.get_finished_orders().await?;

    if orders.is_empty() {
        term.write_line("There are no finished orders")?;
        wait_for_continue(term)?;
        return Ok(());
    }

    for order in orders {
        term.write_line(&order)?;
    }
    wait_for_continue(term)?;

    Ok(())
}

async fn register_report(term: &Term, backend: &ShopBackend) -> Result<()> {
    let order_id = loop {
        term.write_line("Register order")?;
        let order_id: String = Input::new()
            .with_prompt("Order ID (or nothing to go back)")
            .default("-1".to_string())
            .interact_text_on(term)?;
        match order_id.parse::<i32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                continue;
            }
        }
    };

    if order_id == -1 {
        return Ok(());
    }

    if let Err(e) = backend.check_order_id(order_id).await {
        match e.downcast_ref::<DbError>() {
            Some(DbError(s)) => {
                term.write_line(s)?;
                wait_for_continue(term)?;
                return Ok(());
            }
            None => bail!(e),
        }
    }

    let cost = loop {
        let order_id: String = Input::new().with_prompt("Cost").interact_text_on(term)?;
        match order_id.parse::<i32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                continue;
            }
        }
    };

    backend.register_report(order_id, cost).await?;
    term.write_line(&format!("Report on {order_id} has been registered"))?;
    wait_for_continue(term)?;

    Ok(())
}
