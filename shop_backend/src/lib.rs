mod entities;
mod error;
mod migrator;
mod user;

use anyhow::{anyhow, bail, Result};
use function_name::named;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};
use sea_orm_migration::prelude::*;
use serde_json::{self};

use std::env;

use entities::client::{self, Car};
use entities::{order, order::Service, prelude::Order, prelude::*, report};
pub use error::*;
use migrator::Migrator;

pub use user::*;

static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap());

pub struct ShopBackend {
    db: DatabaseConnection,
    user: User,
}

impl ShopBackend {
    /// If SHOP_DATABASE_PATH environment variable exists, backend will use that database,
    /// otherwise ./database.db is used
    pub async fn init() -> Result<Self> {
        let db = Self::connect().await?;

        Ok(ShopBackend {
            db,
            user: User::not_logged_in(),
        })
    }

    async fn connect() -> Result<DatabaseConnection> {
        let database_path = env::var("SHOP_DATABASE_PATH").unwrap_or(String::from("./database.db"));
        let database_url = format!("sqlite:{database_path}?mode=rwc");
        let db = Database::connect(database_url).await?;

        if matches!(db, DatabaseConnection::Disconnected) {
            bail!("database disconnected");
        }

        Migrator::up(&db, None).await?;

        Ok(db)
    }

    pub async fn client_login(&mut self, email: &str, password: &str) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");

        if !EMAIL_REGEX.is_match(email) {
            bail!(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        match Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            Some(client) => {
                if client.password != *password {
                    bail!(anyhow!(LoginError::new(&format!(
                        "incorrect password for {email}"
                    ))));
                }
                self.user = User::logged_in(client.id, &client.name, UserType::Client);
                Ok(self.user.clone())
            }
            None => bail!(DbError(format!("no user with email {email}"))),
        }
    }

    pub async fn employee_login(&mut self, id: i32, password: &str) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");
        match Employee::find_by_id(id).one(&self.db).await? {
            Some(employee) => {
                if employee.password != *password {
                    bail!(LoginError::new(&format!(
                        "incorrect password for employee {id}"
                    )));
                }

                match employee.role {
                    entities::employee::Role::Technician => {
                        self.user =
                            User::logged_in(employee.id, &employee.name, UserType::Technician)
                    }
                    entities::employee::Role::Mechanic => {
                        self.user = User::logged_in(employee.id, &employee.name, UserType::Mechanic)
                    }
                }

                Ok(self.user.clone())
            }
            None => bail!(DbError(format!("employee {id} does not exist"))),
        }
    }

    pub async fn log_out(&mut self) -> User {
        self.user = User::not_logged_in();
        self.user.clone()
    }

    pub fn is_logged_in(&self) -> bool {
        !matches!(self.user.user_type(), UserType::NotLoggedIn)
    }

    fn login_check(&self, func_name: &str) -> Result<()> {
        if !self.is_logged_in() {
            bail!(NotLoggedInError::new(func_name));
        }
        Ok(())
    }

    pub async fn register_client(
        &mut self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<User> {
        assert!(
            !self.is_logged_in(),
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
            password: Set(password.to_owned()),
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

    #[named]
    pub async fn register_order(&self, client_id: i32, service: &Service) -> Result<()> {
        self.login_check(function_name!())?;
        if matches!(self.user.user_type(), UserType::Technician) {
            bail!(PermissionError);
        }

        let Some(client) = Client::find_by_id(client_id).one(&self.db).await? else {
            bail!(DbError(format!("client {client_id} does not exist")));
        };

        match client.car {
            Some(_) => {
                let order = order::ActiveModel {
                    client_id: Set(client_id),
                    service: Set(service.to_owned()),
                    ..Default::default()
                };
                order.insert(&self.db).await?;
                Ok(())
            }
            None => bail!("client {client_id} has no car registered"),
        }
    }

    pub async fn get_unfinished_orders(&self) -> Result<Vec<String>> {
        match self.user.user_type() {
            UserType::Mechanic => {
                let orders = Order::find()
                    .filter(order::Column::Finished.eq(false))
                    .all(&self.db)
                    .await?;
                Ok(orders
                    .iter()
                    .map(|m: &order::Model| serde_json::to_string(m).unwrap())
                    .collect())
            }
            _ => bail!(PermissionError),
        }
    }

    pub async fn get_finished_orders(&self) -> Result<Vec<String>> {
        match self.user.user_type() {
            UserType::Technician => {
                let orders = Order::find()
                    .filter(order::Column::Finished.eq(true))
                    .all(&self.db)
                    .await?;
                Ok(orders
                    .iter()
                    .map(|m| serde_json::to_string(m).unwrap())
                    .collect())
            }
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn change_inspection_to_repair(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!())?;
        if let UserType::Mechanic = self.user.user_type() {
            match Order::find_by_id(order_id).one(&self.db).await? {
                Some(order) => match &order.service {
                    order::Service::Inspection => {
                        let mut order: order::ActiveModel = order.into();
                        order.service = Set(order::Service::Repair);
                        order.update(&self.db).await?;
                        Ok(())
                    }
                    _ => bail!("service to be performed was not inspection"),
                },
                None => bail!(DbError(format!("order {order_id} does not exist"))),
            }
        } else {
            bail!(PermissionError);
        }
    }

    #[named]
    pub async fn close_order(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!())?;
        if let UserType::Mechanic = self.user.user_type() {
            match Order::find_by_id(order_id).one(&self.db).await? {
                Some(order) => {
                    let mut order: order::ActiveModel = order.into();
                    order.finished = Set(true);
                    order.update(&self.db).await?;
                }
                None => bail!(DbError(format!("order {order_id} does not exist"))),
            }
        } else {
            bail!(PermissionError);
        }
        Ok(())
    }

    #[named]
    pub async fn register_report(&self, order_id: i32, cost: i32) -> Result<()> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Technician => match Order::find_by_id(order_id).one(&self.db).await? {
                Some(order) => {
                    let report = report::ActiveModel {
                        client_id: Set(order.client_id),
                        order_id: Set(order_id),
                        cost: Set(cost),
                        ..Default::default()
                    };
                    report.insert(&self.db).await?;
                    Ok(())
                }
                None => bail!(DbError(format!("order {order_id} does not exist"))),
            },
            _ => bail!(""),
        }
    }
}
