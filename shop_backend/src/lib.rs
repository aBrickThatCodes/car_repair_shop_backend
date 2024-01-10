mod database;
mod entities;
mod error;
mod migrator;
mod user;

pub use entities::order::Service;
pub use error::*;
pub use user::*;

use database::ShopDb;
use entities::client::{self, Car};
use entities::{order, report};

use anyhow::{anyhow, bail, Result};
use function_name::named;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::Set;

static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap());

static FINISHED_REPLACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r", finished: true").unwrap());

pub struct ShopBackend {
    db: ShopDb,
    user: User,
}

impl ShopBackend {
    /// If SHOP_DATABASE_PATH environment variable exists, backend will use that database,
    /// otherwise ./database.db is used
    pub async fn init() -> Result<Self> {
        let db = ShopDb::connect().await?;

        Ok(ShopBackend {
            db,
            user: User::not_logged_in(),
        })
    }

    pub async fn client_login(&mut self, email: &str, password: &str) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");

        if !EMAIL_REGEX.is_match(email) {
            bail!(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        match self.db.get_client_by_email(email).await? {
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
        match self.db.get_employee_by_id(id).await? {
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

        if self.db.get_client_by_email(email).await?.is_some() {
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
        self.db.register_client(client).await?;
        let client = self.db.get_client_by_email(email).await?.unwrap();
        self.user = User::logged_in(client.id, name, UserType::Client);
        Ok(self.user.clone())
    }

    #[named]
    pub async fn register_car(&self, client_id: i32, make: &str, model: &str) -> Result<()> {
        self.login_check(function_name!())?;
        if matches!(self.user.user_type(), UserType::Mechanic { .. }) {
            bail!(PermissionError)
        }

        match self.db.get_client_by_id(client_id).await? {
            Some(client) => match &client.car {
                Some(_) => bail!("client already has a car registered"),
                None => {
                    let mut client: client::ActiveModel = client.into();
                    client.car = Set(Some(Car {
                        make: make.to_owned(),
                        model: model.to_owned(),
                    }));
                    self.db.update_client(client).await?;
                    Ok(())
                }
            },
            None => bail!(DbError(format!("client {client_id} does not exits"))),
        }
    }

    #[named]
    pub async fn get_car(&self, client_id: i32) -> Result<Option<String>> {
        self.login_check(function_name!())?;

        match self.db.get_client_by_id(client_id).await? {
            Some(client) => Ok(client.car.map(|car| format!("{car}"))),
            None => unreachable!(),
        }
    }

    #[named]
    pub async fn get_report_summary(&self, report_id: i32) -> Result<String> {
        static ORDER_REPLACE_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"order_id: \d+").unwrap());

        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client { .. } => match self.db.get_report_by_id(report_id).await? {
                Some(report) => {
                    let order_id = report.order_id;
                    let order = self.db.get_order_by_id(order_id).await?.unwrap();
                    Ok(String::from(
                        ORDER_REPLACE_REGEX.replace(&format!("{report}"), format!("{order}")),
                    ))
                }
                None => bail!(DbError(format!("report {report_id} does not exist"))),
            },
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn get_client_orders(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => {
                let orders = self.db.get_clients_orders(self.user.id()).await?;
                Ok(orders
                    .into_iter()
                    .map(|r| format!("{r:?}").replace("Model", "Order"))
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
                let reps = self.db.get_clients_reports(self.user.id()).await?;
                Ok(reps
                    .into_iter()
                    .map(|r| format!("{r:?}").replace("Model", "Report"))
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

        let Some(client) = self.db.get_client_by_id(client_id).await? else {
            bail!(DbError(format!("client {client_id} does not exits")));
        };

        match client.car {
            Some(_) => {
                let order = order::ActiveModel {
                    client_id: Set(client_id),
                    service: Set(service.to_owned()),
                    ..Default::default()
                };
                self.db.register_order(order).await?;
                Ok(())
            }
            None => bail!("client {client_id} has no car registered"),
        }
    }

    #[named]
    pub async fn check_order_id(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!())?;

        match self.db.get_client_by_id(order_id).await? {
            Some(_) => Ok(()),
            None => bail!(DbError(format!("order {order_id} does not exits"))),
        }
    }

    pub async fn get_unfinished_orders(&self) -> Result<Vec<String>> {
        match self.user.user_type() {
            UserType::Mechanic => {
                let orders = self.db.get_orders().await?;
                Ok(orders
                    .iter()
                    .filter(|m| !m.finished)
                    .map(|s| String::from(FINISHED_REPLACE_REGEX.replace(&format!("{s:?}"), "")))
                    .collect())
            }
            _ => bail!(PermissionError),
        }
    }

    pub async fn get_finished_orders(&self) -> Result<Vec<String>> {
        match self.user.user_type() {
            UserType::Mechanic => {
                let orders = self.db.get_orders().await?;
                Ok(orders
                    .iter()
                    .filter(|m| m.finished)
                    .map(|s| String::from(FINISHED_REPLACE_REGEX.replace(&format!("{s:?}"), "")))
                    .collect())
            }
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn change_inspection_to_repair(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!())?;
        if let UserType::Mechanic = self.user.user_type() {
            match self.db.get_order_by_id(order_id).await? {
                Some(order) => match &order.service {
                    order::Service::Inspection => {
                        let mut order: order::ActiveModel = order.into();
                        order.service = Set(order::Service::Repair);
                        self.db.update_order(order).await?;
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
            match self.db.get_order_by_id(order_id).await? {
                Some(order) => {
                    let mut order: order::ActiveModel = order.into();
                    order.finished = Set(true);
                    self.db.update_order(order).await?;
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
            UserType::Technician => match self.db.get_order_by_id(order_id).await? {
                Some(order) => {
                    let report = report::ActiveModel {
                        client_id: Set(order.client_id),
                        order_id: Set(order_id),
                        cost: Set(cost),
                        ..Default::default()
                    };
                    self.db.register_report(report).await?;
                    Ok(())
                }
                None => bail!(DbError(format!("order {order_id} does not exist"))),
            },
            _ => bail!(""),
        }
    }
}
