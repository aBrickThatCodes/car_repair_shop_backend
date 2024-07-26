use anyhow::Result;
use car_repair_shop_backend::*;
use dialoguer::{console::Term, *};

use crate::common::*;

pub async fn mechanic_loop(term: &Term, backend: &mut ShopBackend, user: &User) -> Result<()> {
    static MECHANIC_OPTIONS: [&str; 4] = [
        "List unfinished orders",
        "Change inspection to repair",
        "Close order",
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
            .items(&MECHANIC_OPTIONS)
            .default(0)
            .interact()?;
        term.clear_screen()?;

        match choice {
            0 => list_unfinished_orders(term, backend).await?,
            1 => change_inspection_to_repair(term, backend).await?,
            2 => close_order(term, backend).await?,
            3 => {
                backend.log_out().await?;
                break Ok(());
            }
            _ => unreachable!(),
        }
    }
}

async fn list_unfinished_orders(term: &Term, backend: &ShopBackend) -> Result<()> {
    term.write_line("List reports")?;
    let orders = backend.get_unfinished_orders().await?;

    if orders.is_empty() {
        term.write_line("There are no unfinished orders")?;
        wait_for_continue(term)?;
        return Ok(());
    }

    for order in orders {
        term.write_line(&order)?;
    }
    wait_for_continue(term)?;

    Ok(())
}

async fn change_inspection_to_repair(term: &Term, backend: &ShopBackend) -> Result<()> {
    let order_id = loop {
        term.write_line("Change inspection to repair")?;
        let order_id: String = Input::new()
            .with_prompt("Report ID (or nothing to go back)")
            .default("0".to_string())
            .interact_text_on(term)?;
        match order_id.parse::<u32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format_err(&e))?;
                wait_for_continue(term)?;
                continue;
            }
        }
    };

    if order_id == 0 {
        return Ok(());
    }

    match backend.change_inspection_to_repair(order_id).await {
        Ok(_) => term.write_line(&format!(
            "Order {order_id} changed from inspection to repair"
        ))?,
        Err(e) => term.write_line(&format_err(&e))?,
    }

    wait_for_continue(term)?;
    Ok(())
}

async fn close_order(term: &Term, backend: &ShopBackend) -> Result<()> {
    let order_id = loop {
        term.write_line("Close order")?;
        let order_id: String = Input::new()
            .with_prompt("Order ID (or nothing to go back)")
            .default("0".to_string())
            .interact_text_on(term)?;
        match order_id.parse::<u32>() {
            Ok(i) => break i,
            Err(e) => {
                term.write_line(&format_err(&e))?;
                wait_for_continue(term)?;
                continue;
            }
        }
    };

    if order_id == 0 {
        return Ok(());
    }

    match backend.close_order(order_id).await {
        Ok(_) => term.write_line(&format!("Order {order_id} closed"))?,
        Err(e) => term.write_line(&format_err(&e))?,
    }

    wait_for_continue(term)?;
    Ok(())
}
