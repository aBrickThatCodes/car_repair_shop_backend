use std::env;

use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
};
use sea_orm_migration::prelude::*;

use crate::entities::prelude::Order;
use crate::entities::{prelude::*, *};
use crate::migrator::Migrator;

const DEFAULT_DATABASE_PATH: &str = "./database.db";

pub struct ShopDb {
    db: DatabaseConnection,
}

impl ShopDb {
    /// Connect to the database or crash
    pub async fn connect() -> Result<Self> {
        let database_path =
            env::var("SHOP_DATABASE_PATH").unwrap_or(DEFAULT_DATABASE_PATH.to_string());
        let database_url = format!("sqlite:{database_path}?mode=rwc");
        let db = Database::connect(database_url).await?;

        Migrator::up(&db, None).await?;

        Ok(ShopDb { db })
    }

    pub async fn get_client_by_id(&self, id: i32) -> Result<Option<client::Model>> {
        let v = Client::find_by_id(id).one(&self.db).await?;
        Ok(v)
    }

    pub async fn get_client_by_email(&self, email: &String) -> Result<Option<client::Model>> {
        let v = Client::find()
            .filter(client::Column::Email.eq(email))
            .one(&self.db)
            .await?;
        Ok(v)
    }

    pub async fn register_client(&self, client: client::ActiveModel) -> Result<()> {
        Client::insert(client).exec(&self.db).await?;
        Ok(())
    }

    pub async fn update_client(&self, client: client::ActiveModel) -> Result<()> {
        let _ = client.update(&self.db).await?;
        Ok(())
    }

    //Employee functions
    pub async fn get_employee_by_id(&self, id: i32) -> Result<Option<employee::Model>> {
        let v = Employee::find_by_id(id).one(&self.db).await?;
        Ok(v)
    }

    // order functions
    /// Get all standing orders
    pub async fn get_standing_orders(&self) -> Result<Vec<order::Model>> {
        let v = Order::find()
            .filter(order::Column::Finished.eq(false))
            .all(&self.db)
            .await?;
        Ok(v)
    }

    pub async fn get_clients_orders(&self, client_id: i32) -> Result<Vec<order::Model>> {
        let v = Order::find()
            .filter(order::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await?;
        Ok(v)
    }

    pub async fn get_order_by_id(&self, id: i32) -> Result<Option<order::Model>> {
        let v = Order::find_by_id(id).one(&self.db).await?;
        Ok(v)
    }

    pub async fn register_order(&self, order: order::ActiveModel) -> Result<()> {
        Order::insert(order).exec(&self.db).await?;
        Ok(())
    }

    pub async fn update_order(&self, order: order::ActiveModel) -> Result<()> {
        order.update(&self.db).await?;
        Ok(())
    }

    // Report functions
    pub async fn get_report_by_id(&self, id: i32) -> Result<Option<report::Model>> {
        let v = Report::find_by_id(id).one(&self.db).await?;
        Ok(v)
    }

    pub async fn get_clients_reports(&self, client_id: i32) -> Result<Vec<report::Model>> {
        let v = Report::find()
            .filter(report::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await?;
        Ok(v)
    }

    pub async fn register_report(&self, report: report::ActiveModel) -> Result<()> {
        Report::insert(report).exec(&self.db).await?;
        Ok(())
    }
}
