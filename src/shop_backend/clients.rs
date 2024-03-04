use super::*;
use crate::{db_entities::client, UserType, *};

use function_name::named;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};
use sea_orm_migration::prelude::*;

impl ShopBackend {
    pub async fn client_login(
        &mut self,
        email: &str,
        password_hash: &str,
    ) -> Result<User, LoginError> {
        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            return Err(LoginError::AlreadyLoggedIn);
        };

        match db_entities::prelude::Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await
        {
            Ok(res) => match res {
                Some(client) => {
                    if !EMAIL_REGEX.is_match(email) {
                        return Err(LoginError::EmailIncorrectFormat(email.to_owned()));
                    }

                    if !HASH_REGEX.is_match(password_hash) {
                        return Err(LoginError::PasswordNotHashed);
                    }

                    if client.password_hash != *password_hash {
                        return Err(LoginError::ClientIncorrectPassword(email.to_string()));
                    }

                    self.user = User::logged_in(client.id as u32, &client.name, UserType::Client);
                    Ok(self.user.clone())
                }
                None => Err(LoginError::EmailNotRegistered(email.to_string())),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn register_client(
        &mut self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, RegisterClientError> {
        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            return Err(RegisterClientError::AlreadyLoggedIn);
        }

        if !EMAIL_REGEX.is_match(email) {
            return Err(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        match db_entities::prelude::Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            Some(_) => Err(RegisterClientError::EmailAlreadyRegistered(
                email.to_owned(),
            )),
            None => {
                let client = client::ActiveModel {
                    name: Set(name.to_owned()),
                    email: Set(email.to_owned()),
                    password_hash: Set(password_hash.to_string()),
                    ..Default::default()
                };
                let res = client.insert(&self.db).await?;
                self.user = User::logged_in(res.id as u32, name, UserType::Client);
                Ok(self.user.clone())
            }
        }
    }

    #[named]
    pub async fn get_car(&self, client_id: u32) -> Result<Option<Car>, DbError> {
        self.login_check(function_name!())?;

        match db_entities::prelude::Client::find_by_id(client_id as i32)
            .one(&self.db)
            .await?
        {
            Some(client) => Ok(client.car.clone()),
            None => Err(DbError::Client(client_id)),
        }
    }

    #[named]
    pub async fn get_client_orders(&self) -> Result<Vec<crate::Order>, DbError> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let client = db_entities::prelude::Client::find_by_id(self.user.id() as i32)
                    .one(&self.db)
                    .await?
                    .unwrap();
                let orders = client
                    .find_related(db_entities::prelude::Order)
                    .all(&self.db)
                    .await?;
                Ok(orders.into_iter().map(|m| m.into()).collect())
            }
            _ => Err(DbError::Permission),
        }
    }

    #[named]
    pub async fn get_client_reports(&self) -> Result<Vec<Report>, DbError> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let client = db_entities::prelude::Client::find_by_id(self.user.id() as i32)
                    .one(&self.db)
                    .await?
                    .unwrap();
                let reps = client
                    .find_related(db_entities::prelude::Report)
                    .all(&self.db)
                    .await?;
                Ok(reps.into_iter().map(|m| m.into()).collect())
            }
            _ => Err(DbError::Permission),
        }
    }
}
