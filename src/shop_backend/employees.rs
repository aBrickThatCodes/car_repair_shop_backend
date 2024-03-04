use super::HASH_REGEX;

use crate::db_entities::employee;
use crate::UserType;
use crate::*;

use sea_orm::EntityTrait;

impl ShopBackend {
    pub async fn employee_login(
        &mut self,
        id: u32,
        password_hash: &str,
    ) -> Result<User, LoginError> {
        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            return Err(LoginError::AlreadyLoggedIn);
        };

        if !HASH_REGEX.is_match(password_hash) {
            return Err(LoginError::PasswordNotHashed);
        }

        match db_entities::prelude::Employee::find_by_id(id as i32)
            .one(&self.db)
            .await?
        {
            Some(employee) => {
                if employee.password_hash != *password_hash {
                    return Err(LoginError::EmployeeIncorrectPassword(id));
                }

                match employee.role {
                    employee::Role::Technician => {
                        self.user = User::logged_in(id, &employee.name, UserType::Technician)
                    }
                    employee::Role::Mechanic => {
                        self.user = User::logged_in(id, &employee.name, UserType::Mechanic)
                    }
                }

                Ok(self.user.clone())
            }
            None => Err(LoginError::EmployeeNotRegistered(id)),
        }
    }
}
