use super::common::HASH_REGEX;

use crate::entities::{employee, prelude::*};
use crate::UserType;
use crate::*;

use anyhow::{bail, Result};
use sea_orm::EntityTrait;
use zeroize::{Zeroize, Zeroizing};

impl ShopBackend {
    pub async fn employee_login(&mut self, id: i32, password_hash: &mut str) -> Result<User> {
        let password_hash_zeroing = Zeroizing::new(password_hash.to_string());
        password_hash.zeroize();

        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            bail!(LoginError::AlreadyLoggedIn);
        };

        if !HASH_REGEX.is_match(&password_hash_zeroing) {
            bail!(LoginError::PasswordNotHashed)
        }

        match Employee::find_by_id(id).one(&self.db).await? {
            Some(employee) => {
                if employee.password != *password_hash_zeroing {
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
