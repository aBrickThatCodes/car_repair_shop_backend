use crate::db_entities::prelude::Order;
use crate::db_entities::{self, report};
use crate::{DbError, ShopBackend};
use crate::{Report, UserType};

use function_name::named;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

impl ShopBackend {
    #[named]
    pub async fn get_report(&self, report_id: u32) -> Result<Report, DbError> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Client => match db_entities::prelude::Report::find_by_id(report_id as i32)
                .one(&self.db)
                .await
            {
                Ok(res) => match res {
                    Some(m) => Ok::<Report, DbError>(m.into()),
                    None => Err(DbError::Report(report_id)),
                },
                Err(e) => Err(e.into()),
            },
            _ => todo!(),
        }
    }

    #[named]
    pub async fn register_report(&self, order_id: u32, cost: u32) -> Result<(), DbError> {
        self.login_check(function_name!())?;
        match self.user.user_type() {
            UserType::Technician => match Order::find_by_id(order_id as i32).one(&self.db).await? {
                Some(order) => {
                    let report = report::ActiveModel {
                        client_id: Set(order.client_id),
                        order_id: Set(order_id as i32),
                        cost: Set(cost as i32),
                        ..Default::default()
                    };
                    report.insert(&self.db).await?;
                    Ok(())
                }
                None => Err(DbError::Order(order_id)),
            },
            _ => Err(DbError::Permission),
        }
    }
}
