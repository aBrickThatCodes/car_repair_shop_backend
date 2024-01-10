use super::ShopBackend;

use super::common::*;
use crate::{
    entities::{
        client::{self, Car},
        prelude::*,
    },
    error::*,
    User, UserType,
};

use anyhow::{bail, Result};
use function_name::named;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};
use sea_orm_migration::prelude::*;

impl ShopBackend {
    pub async fn client_login(&mut self, email: &str, password_hash: &str) -> Result<User> {
        assert!(
            matches!(self.user.user_type(), UserType::NotLoggedIn),
            "already logged in"
        );

        if !EMAIL_REGEX.is_match(email) {
            bail!(LoginError::EmailIncorrectFormat(email.to_owned()));
        }

        if !HASH_REGEX.is_match(password_hash) {
            bail!(LoginError::PasswordNotHashed)
        }

        match Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            Some(client) => {
                if client.password_hash != *password_hash {
                    bail!(LoginError::ClientIncorrectPassword(email.to_string()));
                }
                self.user = User::logged_in(client.id, &client.name, UserType::Client);
                Ok(self.user.clone())
            }
            None => bail!(LoginError::EmailNotRegistered(email.to_string())),
        }
    }

    pub async fn register_client(
        &mut self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        assert!(
            matches!(self.user.user_type(), UserType::NotLoggedIn),
            "cannot register a client if already logged in"
        );

        if !EMAIL_REGEX.is_match(email) {
            bail!(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        if Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
            .is_some()
        {
            bail!(RegisterClientError::EmailAlreadyRegistered(
                email.to_owned()
            ));
        }

        let client = client::ActiveModel {
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(password_hash.to_owned()),
            ..Default::default()
        };
        let res = client.insert(&self.db).await?;
        self.user = User::logged_in(res.id, name, UserType::Client);
        Ok(self.user.clone())
    }

    #[named]
    pub async fn register_car(&self, client_id: i32, make: &str, model: &str) -> Result<()> {
        self.login_check(function_name!())?;
        if matches!(self.user.user_type(), UserType::Mechanic { .. }) {
            bail!(PermissionError)
        }

        match Client::find_by_id(client_id).one(&self.db).await? {
            Some(client) => match &client.car {
                Some(_) => bail!("client already has a car registered"),
                None => {
                    let mut client: client::ActiveModel = client.into();
                    client.car = Set(Some(Car {
                        make: make.to_owned(),
                        model: model.to_owned(),
                    }));
                    client.update(&self.db).await?;
                    Ok(())
                }
            },
            None => bail!(DbError(format!("client {client_id} does not exist"))),
        }
    }

    #[named]
    pub async fn get_car(&self, client_id: i32) -> Result<Option<String>> {
        self.login_check(function_name!())?;

        match Client::find_by_id(client_id).one(&self.db).await? {
            Some(client) => Ok(client.car.map(|car| serde_json::to_string(&car).unwrap())),
            None => unreachable!(),
        }
    }

    #[named]
    pub async fn get_client_orders(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let client = Client::find_by_id(self.user.id())
                    .one(&self.db)
                    .await?
                    .unwrap();
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
                let client = Client::find_by_id(self.user.id())
                    .one(&self.db)
                    .await?
                    .unwrap();
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
