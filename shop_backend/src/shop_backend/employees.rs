use super::HASH_REGEX;

use crate::db_entities::employee;
use crate::UserType;
use crate::*;

use anyhow::{bail, Result};
use sea_orm::EntityTrait;

impl ShopBackend {
    pub async fn employee_login(&mut self, id: i32, password_hash: &str) -> Result<User> {
        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            bail!(LoginError::AlreadyLoggedIn);
        };

        if !HASH_REGEX.is_match(password_hash) {
            bail!(LoginError::PasswordNotHashed)
        }

        match db_entities::prelude::Employee::find_by_id(id)
            .one(&self.db)
            .await?
        {
            Some(employee) => {
                if employee.password_hash != *password_hash {
                    bail!(LoginError::EmployeeIncorrectPassword(id));
                }

                match employee.role {
                    employee::Role::Technician => {
                        self.user =
                            User::logged_in(employee.id, &employee.name, UserType::Technician)
                    }
                    employee::Role::Mechanic => {
                        self.user = User::logged_in(employee.id, &employee.name, UserType::Mechanic)
                    }
                }

                Ok(self.user.clone())
            }
            None => bail!(LoginError::EmployeeNotRegistered(id)),
        }
    }
}
