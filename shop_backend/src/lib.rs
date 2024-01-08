mod database;
mod entities;
mod errors;
mod migrator;

pub use entities::order::Service;
pub use errors::*;

use database::ShopDb;
use entities::client::{self, Car};
use entities::{order, report};

use anyhow::{anyhow, bail, Result};
use function_name::named;
use sea_orm::Set;

#[derive(Clone)]
pub enum User {
    Client {
        id: i32,
        email: String,
        name: String,
    },
    Technician {
        id: i32,
        name: String,
    },
    Mechanic {
        id: i32,
        name: String,
    },
    NotLoggedIn,
}

pub struct ShopBackend {
    db: ShopDb,
    user: User,
}

impl ShopBackend {
    pub async fn init() -> Self {
        let db = ShopDb::connect().await;

        ShopBackend {
            db,
            user: User::NotLoggedIn,
        }
    }

    pub async fn client_login(&mut self, email: &str, password: &str) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");
        match self.db.get_client_by_email(email).await? {
            Some(client) => {
                if client.password != password {
                    bail!(anyhow!(LoginError::new(format!(
                        "incorrect password for {email}"
                    ))));
                }
                self.user = User::Client {
                    id: client.id,
                    email: email.to_string(),
                    name: client.name,
                };
                Ok(self.user.clone())
            }
            None => bail!(DbError::new(format!("no user with email {email}"))),
        }
    }

    pub async fn employee_login(&mut self, id: i32, password: &str) -> Result<User> {
        assert!(!self.is_logged_in(), "already logged in");
        match self.db.get_employee_by_id(id).await? {
            Some(employee) => {
                if employee.password != password {
                    bail!(anyhow!(LoginError::new(format!(
                        "incorrect password for employee {id}"
                    ))));
                }

                match employee.role {
                    entities::employee::Role::Technician => {
                        self.user = User::Technician {
                            id,
                            name: String::new(),
                        }
                    }
                    entities::employee::Role::Mechanic => {
                        self.user = User::Mechanic {
                            id,
                            name: String::new(),
                        }
                    }
                }

                Ok(self.user.clone())
            }
            None => bail!(DbError::new(format!("employee {id} does not exist"))),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        !matches!(self.user, User::NotLoggedIn)
    }

    fn login_check(&self, func_name: &str) {
        assert!(self.is_logged_in(), "{func_name} requires being logged in")
    }

    pub async fn register_user(&mut self, name: &str, email: &str, password: &str) -> Result<User> {
        assert!(
            !self.is_logged_in(),
            "cannot register a client if already logged in"
        );

        if self.db.get_client_by_email(email).await?.is_none() {
            bail!(RegisterClientError);
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
            email: email.to_string(),
            id: client.id,
            name: name.to_string(),
        };
        Ok(self.user.clone())
    }

    #[named]
    pub async fn get_user_client_id(&self) -> Result<i32> {
        self.login_check(function_name!());
        match &self.user {
            User::Client {
                email: _,
                id,
                name: _,
            } => Ok(*id),
            _ => bail!("not logged in as client"),
        }
    }

    #[named]
    pub async fn get_user_orders(&self) -> Result<Vec<String>> {
        self.login_check(function_name!());
        match &self.user {
            User::Client {
                email: _,
                id: _,
                name: _,
            } => {
                let id = self.get_user_client_id().await?;
                let orders = self.db.get_clients_orders(id).await?;
                Ok(orders.into_iter().map(|r| format!("{r:?}")).collect())
            }
            _ => bail!("not logged in as a client"),
        }
    }

    #[named]
    pub async fn get_user_reports(&self) -> Result<Vec<String>> {
        self.login_check(function_name!());
        match &self.user {
            User::Client { .. } => {
                let id = self.get_user_client_id().await?;
                let reps = self.db.get_clients_reports(id).await?;
                Ok(reps.into_iter().map(|r| format!("{r:?}")).collect())
            }
            _ => bail!("not logged in as a client"),
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
    pub async fn register_vehicle(
        &self,
        client_id: i32,
        make: String,
        model: String,
    ) -> Result<()> {
        self.login_check(function_name!());
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
            None => bail!(DbError::new(String::from(
                "client {client_id} does not exits"
            ))),
        }
    }

    #[named]
    pub async fn client_create_order(&self, service: Service) -> Result<()> {
        self.login_check(function_name!());

        match self.user {
            User::Client { id, .. } => {
                let Some(client) = self.db.get_client_by_id(id).await? else {
                    unreachable!()
                };

                match client.car {
                    Some(_) => {
                        let order = order::ActiveModel {
                            client_id: Set(id),
                            service: Set(service),
                            ..Default::default()
                        };
                        self.db.register_order(order).await?;
                        Ok(())
                    }
                    None => bail!("client {id} has no car registered"),
                }
            }
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn technician_create_order(&self, client_id: i32, service: Service) -> Result<()> {
        self.login_check(function_name!());

        match self.user {
            User::Technician { .. } => {
                let Some(client) = self.db.get_client_by_id(client_id).await? else {
                    unreachable!()
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
            _ => bail!(PermissionError),
        }
    }

    #[named]
    pub async fn change_inspection_to_repair(&self, order_id: i32) -> Result<()> {
        self.login_check(function_name!());
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
                None => bail!(DbError::new(String::from(
                    "order {order_id} does not exist"
                ))),
            }
        } else {
            bail!(PermissionError);
        }
    }

    #[named]
    pub async fn create_report(&self, order_id: i32, cost: i32) -> Result<()> {
        self.login_check(function_name!());
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
                None => bail!(DbError::new(String::from(
                    "order {order_id} does not exist"
                ))),
            },
            _ => bail!(""),
        }
    }

    #[named]
    pub async fn get_report_summary(&self, report_id: i32) -> Result<String> {
        self.login_check(function_name!());
        match self.user {
            User::Client { .. } => match self.db.get_report_by_id(report_id).await? {
                Some(report) => {
                    let order_id = report.order_id;
                    let order = self.db.get_order_by_id(order_id).await?.unwrap();
                    Ok(format!("{report:#?}\n{order:#?}"))
                }
                None => bail!(DbError::new(format!("report {report_id} does not exist"))),
            },
            _ => bail!(PermissionError),
        }
    }
}
