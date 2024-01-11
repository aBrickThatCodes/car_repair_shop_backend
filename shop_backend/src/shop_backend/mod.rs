mod clients;
mod employees;
mod orders;
mod reports;

use super::db_entities::client::{self, Car};
use super::migrator::Migrator;
use super::{user::*, *};

use anyhow::{bail, Result};
use function_name::named;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Set, Statement,
};
use sea_orm_migration::prelude::*;
use std::env;

pub static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap());

pub static HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\$2[aby]?\$\d{1,2}\$[./A-Za-z0-9]{53}$").unwrap());

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
        let db_url = env::var("SHOP_db_url").unwrap_or(String::from("sqlite:./shop.db?mode=rwc"));
        let db_name = env::var("SHOP_DATABASE_NAME").unwrap_or(String::from("shop"));
        let db = Database::connect(&db_url).await?;

        if matches!(db, DatabaseConnection::Disconnected) {
            bail!("database disconnected");
        }

        let db = match db.get_database_backend() {
            DbBackend::MySql => {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
                ))
                .await?;

                let url = format!("{}/{}", db_url, db_name);
                Database::connect(&url).await?
            }
            DbBackend::Postgres => {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
                ))
                .await?;
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", db_name),
                ))
                .await?;

                let url = format!("{}/{}", db_url, db_name);
                Database::connect(&url).await?
            }
            DbBackend::Sqlite => db,
        };

        Migrator::up(&db, None).await?;

        Ok(db)
    }

    #[named]
    pub async fn log_out(&mut self) -> Result<User> {
        self.login_check(function_name!())?;
        self.user = User::not_logged_in();
        Ok(self.user.clone())
    }

    pub fn login_check(&self, func_name: &str) -> Result<()> {
        if matches!(self.user.user_type(), UserType::NotLoggedIn) {
            func_name.to_string();
        }
        Ok(())
    }

    #[named]
    pub async fn register_car(&self, client_id: u32, make: &str, model: &str) -> Result<()> {
        self.login_check(function_name!())?;

        if matches!(self.user.user_type(), UserType::Mechanic { .. }) {
            bail!(PermissionError)
        }

        match db_entities::prelude::Client::find_by_id(client_id as i32)
            .one(&self.db)
            .await?
        {
            Some(client) => match &client.car {
                Some(_) => bail!("client already has a car registered"),
                None => {
                    let mut client_active: client::ActiveModel = client.into();
                    client_active.car = Set(Some(Car {
                        make: make.to_owned(),
                        model: model.to_owned(),
                    }));
                    client_active.update(&self.db).await?;
                    Ok(())
                }
            },
            None => bail!(DbError::Client(client_id)),
        }
    }
}
