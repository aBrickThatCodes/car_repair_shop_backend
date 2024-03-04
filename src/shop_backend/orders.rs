use crate::{db_entities::order, UserType, *};

use function_name::named;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

impl ShopBackend {
    #[named]
    pub async fn register_order(&self, client_id: u32, service: &Service) -> Result<(), DbError> {
        self.login_check(function_name!())?;
        if matches!(self.user.user_type(), UserType::Technician) {
            return Err(DbError::Permission);
        }

        let Some(client) = db_entities::prelude::Client::find_by_id(client_id as i32)
            .one(&self.db)
            .await?
        else {
            return Err(DbError::Client(client_id));
        };

        match client.car {
            Some(_) => {
                let order = order::ActiveModel {
                    client_id: Set(client_id as i32),
                    service: Set(service.to_owned()),
                    ..Default::default()
                };
                order.insert(&self.db).await?;
                Ok(())
            }
            None => Err(DbError::Other(String::from(
                "client {client_id} has no car registered",
            ))),
        }
    }

    pub async fn get_unfinished_orders(&self) -> Result<Vec<String>, DbError> {
        match self.user.user_type() {
            UserType::Mechanic => {
                let orders = db_entities::prelude::Order::find()
                    .filter(order::Column::Finished.eq(false))
                    .all(&self.db)
                    .await?;
                Ok(orders
                    .iter()
                    .map(|m: &order::Model| serde_json::to_string(m).unwrap())
                    .collect::<Vec<_>>())
            }
            _ => Err(DbError::Permission),
        }
    }

    pub async fn get_finished_orders(&self) -> Result<Vec<Order>, DbError> {
        match self.user.user_type() {
            UserType::Technician => {
                let orders = db_entities::prelude::Order::find()
                    .filter(order::Column::Finished.eq(true))
                    .all(&self.db)
                    .await?;
                Ok(orders.into_iter().map(|m| m.into()).collect::<Vec<Order>>())
            }
            _ => Err(DbError::Permission),
        }
    }

    #[named]
    pub async fn change_inspection_to_repair(&self, order_id: u32) -> Result<(), DbError> {
        self.login_check(function_name!())?;
        if let UserType::Mechanic = self.user.user_type() {
            match db_entities::prelude::Order::find_by_id(order_id as i32)
                .one(&self.db)
                .await?
            {
                Some(order) => match &order.service {
                    order::Service::Inspection => {
                        let mut order: order::ActiveModel = order.into();
                        order.service = Set(order::Service::Repair);
                        order.update(&self.db).await?;
                        Ok(())
                    }
                    _ => Err(DbError::Other(String::from(
                        "service to be performed was not inspection",
                    ))),
                },
                None => Err(DbError::Order(order_id)),
            }
        } else {
            Err(DbError::Permission)
        }
    }

    #[named]
    pub async fn close_order(&self, order_id: u32) -> Result<(), DbError> {
        self.login_check(function_name!())?;
        if let UserType::Mechanic = self.user.user_type() {
            match db_entities::prelude::Order::find_by_id(order_id as i32)
                .one(&self.db)
                .await?
            {
                Some(order) => {
                    let mut order: order::ActiveModel = order.into();
                    order.finished = Set(true);
                    order.update(&self.db).await?;
                }
                None => return Err(DbError::Order(order_id)),
            }
        } else {
            return Err(DbError::Permission);
        }
        Ok(())
    }
}
