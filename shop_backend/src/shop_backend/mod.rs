mod clients;
mod common;
mod employee;
mod orders;
mod reports;

use super::migrator::Migrator;
use super::user::*;

use anyhow::{bail, Result};
use function_name::named;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use std::env;

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

    #[named]
    pub async fn log_out(&mut self) -> Result<User> {
        self.login_check(function_name!())?;
        self.user = User::not_logged_in();
        Ok(self.user.clone())
    }
}
