use super::common::*;
use crate::{
    entities::{client, prelude::*},
    UserType, *,
};

use anyhow::{bail, Result};
use function_name::named;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};
use sea_orm_migration::prelude::*;
use zeroize::{Zeroize, Zeroizing};

impl ShopBackend {
    pub async fn client_login(&mut self, email: &str, password_hash: &mut str) -> Result<User> {
        let password_hash_zeroing = Zeroizing::new(password_hash.to_string());
        password_hash.zeroize();

        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            bail!(LoginError::AlreadyLoggedIn);
        };

        match Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            Some(mut client) => {
                if !EMAIL_REGEX.is_match(email) {
                    client.password_hash.zeroize();
                    bail!(LoginError::EmailIncorrectFormat(email.to_owned()));
                }

                if !HASH_REGEX.is_match(&password_hash_zeroing) {
                    client.password_hash.zeroize();
                    bail!(LoginError::PasswordNotHashed)
                }

                if client.password_hash != *password_hash_zeroing {
                    client.password_hash.zeroize();
                    bail!(LoginError::ClientIncorrectPassword(email.to_string()));
                }

                client.password_hash.zeroize();

                self.user = User::logged_in(client.id, &client.name, UserType::Client);
                Ok(self.user.clone())
            }
            None => {
                bail!(LoginError::EmailNotRegistered(email.to_string()))
            }
        }
    }

    pub async fn register_client(
        &mut self,
        name: &str,
        email: &str,
        password_hash: &mut str,
    ) -> Result<User> {
        let password_hash_zeroing = Zeroizing::new(password_hash.to_string());
        password_hash.zeroize();

        if !matches!(self.user.user_type(), UserType::NotLoggedIn) {
            bail!(RegisterClientError::AlreadyLoggedIn);
        }

        if !EMAIL_REGEX.is_match(email) {
            bail!(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        match Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            Some(mut client) => {
                client.password_hash.zeroize();
                bail!(RegisterClientError::EmailAlreadyRegistered(
                    email.to_owned()
                ))
            }
            None => {
                let client = client::ActiveModel {
                    name: Set(name.to_owned()),
                    email: Set(email.to_owned()),
                    password_hash: Set(password_hash_zeroing.to_string()),
                    ..Default::default()
                };
                let res = client.insert(&self.db).await?;
                self.user = User::logged_in(res.id, name, UserType::Client);
                Ok(self.user.clone())
            }
        }
    }

    #[named]
    pub async fn get_car(&self, client_id: i32) -> Result<Option<String>> {
        self.login_check(function_name!())?;

        match Client::find_by_id(client_id).one(&self.db).await? {
            Some(mut client) => {
                client.password_hash.zeroize();
                Ok(client
                    .car
                    .clone()
                    .map(|car| serde_json::to_string(&car).unwrap()))
            }
            None => unreachable!(),
        }
    }

    #[named]
    pub async fn get_client_orders(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let mut client = Client::find_by_id(self.user.id())
                    .one(&self.db)
                    .await?
                    .unwrap();
                client.password_hash.zeroize();
                let orders = client.find_related(Order).all(&self.db).await?;
                Ok(orders
                    .iter()
                    .map(|m| serde_json::to_string(m).unwrap())
                    .collect())
            }
            _ => bail!("not logged in as a client"),
        }
    }

    #[named]
    pub async fn get_client_reports(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let mut client = Client::find_by_id(self.user.id())
                    .one(&self.db)
                    .await?
                    .unwrap();
                client.password_hash.zeroize();
                let reps = client.find_related(Report).all(&self.db).await?;
                Ok(reps
                    .iter()
                    .map(|m| serde_json::to_string(m).unwrap())
                    .collect())
            }
            _ => bail!("not logged in as a client"),
        }
    }
}
