mod database;
mod entities;
mod error;
mod migrator;
mod user;

pub use entities::order::Service;
pub use error::*;
pub use user::User;

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
            user: User::NotLoggedIn,
        })
    }

    pub async fn client_login(&mut self, email: &String, password: &String) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");

        if !EMAIL_REGEX.is_match(email) {
            bail!(RegisterClientError::EmailIncorrectFormat(email.to_owned()));
        }

        match self.db.get_client_by_email(email).await? {
            Some(client) => {
                if client.password != *password {
                    bail!(anyhow!(LoginError::new(format!(
                        "incorrect password for {email}"
                    ))));
                }
                self.user = User::Client {
                    id: client.id,
                    email: email.to_owned(),
                    name: client.name,
                };
                Ok(self.user.clone())
            }
            None => bail!(DbError::new(format!("no user with email {email}"))),
        }
    }

    pub async fn employee_login(&mut self, id: i32, password: &String) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");
        match self.db.get_employee_by_id(id).await? {
            Some(employee) => {
                if employee.password != *password {
                    bail!(LoginError::new(format!(
                        "incorrect password for employee {id}"
                    )));
                }

                match employee.role {
                    entities::employee::Role::Technician => {
                        self.user = User::Technician {
                            id,
                            name: employee.name,
                        }
                    }
                    entities::employee::Role::Mechanic => {
                        self.user = User::Mechanic {
                            id,
                            name: employee.name,
                        }
                    }
                }

                Ok(self.user.clone())
            }
            None => bail!(DbError::new(format!("employee {id} does not exist"))),
        }
    }

    pub async fn log_out(&mut self) -> User {
        self.user = User::NotLoggedIn;
        User::NotLoggedIn
    }

    pub fn is_logged_in(&self) -> bool {
        !matches!(self.user, User::NotLoggedIn)
    }

    fn login_check(&self, func_name: &str) -> Result<()> {
        if !self.is_logged_in() {
            bail!(NotLoggedInError::new(func_name.to_string()));
        }
        Ok(())
    }

    pub async fn register_client(
        &mut self,
        name: &String,
        email: &String,
        password: &String,
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
        self.user = User::Client {
            email: email.to_owned(),
            id: client.id,
            name: name.to_owned(),
        };
        Ok(self.user.clone())
    }

    #[named]
    pub async fn register_car(&self, client_id: i32, make: String, model: String) -> Result<()> {
        self.login_check(function_name!())?;
        if matches!(self.user, User::Mechanic { .. }) {
            bail!(PermissionError)
        }

        match self.db.get_client_by_id(client_id).await? {
            Some(client) => match &client.car {
                Some(_) => bail!("client already has a car registered"),
                None => {
                    let mut client: client::ActiveModel = client.into();
                    client.car = Set(Some(Car { make, model }));
                    self.db.update_client(client).await?;
                    Ok(())
                }
            },
            None => bail!(DbError::new(format!("client {client_id} does not exits"))),
        }
    }

    #[named]
    pub async fn get_car(&self, client_id: i32) -> Result<Option<String>> {
        self.login_check(function_name!())?;

        match self.db.get_client_by_id(client_id).await? {
            Some(client) => Ok(client.car.map(|car| format!("{car:?}"))),
            None => unreachable!(),
        }
    }

    #[named]
    pub async fn get_report_summary(&self, report_id: i32) -> Result<String> {
        self.login_check(function_name!())?;
        match self.user {
            User::Client { .. } => match self.db.get_report_by_id(report_id).await? {
                Some(report) => {
                    let order_id = report.order_id;
                    let order = self.db.get_order_by_id(order_id).await?.unwrap();

                    let mut report_string = format!("{order:#?}");
                    report_string = report_string
                        .split('\n')
                        .filter(|s| {
                            let s = s.trim_start();
                            !(s.starts_with("client_id") || s.starts_with("order_id"))
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(format!("{report_string}\n{order:#?}"))
                }
                None => bail!(DbError::new(format!("report {report_id} does not exist"))),
            },
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn get_client_orders(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user {
            User::Client { id, .. } => {
                let orders = self.db.get_clients_orders(id).await?;
                Ok(orders.into_iter().map(|r| format!("{r:?}")).collect())
            }
            _ => bail!("not logged in as a client"),
        }
    }

    #[named]
    pub async fn get_client_reports(&self) -> Result<Vec<String>> {
        self.login_check(function_name!())?;
        match self.user {
            User::Client { id, .. } => {
                let reps = self.db.get_clients_reports(id).await?;
                Ok(reps.into_iter().map(|r| format!("{r:?}")).collect())
            }
            _ => bail!("not logged in as a client"),
        }
    }

    #[named]
    pub async fn register_order(&self, client_id: i32, service: Service) -> Result<()> {
        self.login_check(function_name!())?;
        if matches!(self.user, User::Mechanic { .. }) {
            bail!(PermissionError);
        }

        let Some(client) = self.db.get_client_by_id(client_id).await? else {
            bail!(DbError::new(format!("client {client_id} does not exits")));
        };

        match client.car {
            Some(_) => {
                let order = order::ActiveModel {
                    client_id: Set(client_id),
                    service: Set(service),
                    ..Default::default()
                };
                self.db.register_order(order).await?;
                Ok(())
            }
            None => bail!("client {client_id} has no car registered"),
        }
    }

    pub async fn get_standing_orders(&self) -> Result<Vec<String>> {
        match self.user {
            User::Mechanic { .. } => {
                let orders = self.db.get_standing_orders().await?;
                Ok(orders.into_iter().map(|r| format!("{r:?}")).collect())
            }
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn change_inspection_to_repair(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!())?;
        if let User::Mechanic { .. } = self.user {
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
                None => bail!(DbError::new(format!("order {order_id} does not exist"))),
            }
        } else {
            bail!(PermissionError);
        }
    }

    #[named]
    pub async fn create_report(&self, order_id: i32, cost: i32) -> Result<()> {
        self.login_check(function_name!())?;
        match self.user {
            User::Mechanic { .. } => match self.db.get_order_by_id(order_id).await? {
                Some(order) => {
                    let mut order: order::ActiveModel = order.into();
                    order.finished = Set(true);
                    let client_id = order.clone().client_id.unwrap();
                    self.db.update_order(order).await?;
                    let report = report::ActiveModel {
                        client_id: Set(client_id),
                        order_id: Set(order_id),
                        cost: Set(cost),
                        ..Default::default()
                    };
                    self.db.register_report(report).await?;
                    Ok(())
                }
                None => bail!(DbError::new(format!("order {order_id} does not exist"))),
            },
            _ => bail!(""),
        }
    }
}
