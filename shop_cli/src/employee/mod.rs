mod mechanic;
mod technician;

use anyhow::Result;
use console::Term;
use dialoguer::*;
use shop_backend::*;

use crate::common::wait_for_continue;

#[allow(clippy::never_loop)]
pub async fn employee_loop(term: &Term, mut backend: ShopBackend) -> Result<()> {
    loop {
        let user = login_screen(term, &mut backend).await?;
        term.write_line(&format!("{user:?}"))?;
        match &user {
            User::Technician { .. } => todo!("technician_loop"),
            User::Mechanic { .. } => todo!("mechanic_loop"),
            _ => unreachable!(),
        }
    }
}

async fn login_screen(term: &Term, backend: &mut ShopBackend) -> Result<User> {
    loop {
        term.clear_screen()?;

        term.write_line("Car Repair Shop Employee Interface")?;

        let choice = Select::new()
            .items(&["Login", "Exit"][..])
            .default(0)
            .interact_on(term)?;

        if choice == 1 {
            std::process::exit(0);
        }

        term.clear_screen()?;

        term.write_line("Login")?;

        let id: String = Input::new().with_prompt("ID").interact_text_on(term)?;
        let password = Password::new().with_prompt("Password").interact_on(term)?;

        let id = match id.parse::<i32>() {
            Ok(i) => i,
            Err(_) => {
                term.write_line(&format!("{} is not a valid ID", id))?;
                continue;
            }
        };

        let user = backend.employee_login(id, &password).await;

        term.clear_screen()?;
        match user {
            Err(e) => {
                term.write_line(&format!("{e}"))?;
                wait_for_continue(term)?;
                continue;
            }
            Ok(employee) => break Ok(employee),
        }
    }
}
