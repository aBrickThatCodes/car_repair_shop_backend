use crate::db_entities::prelude::Order;
use crate::db_entities::report;
use crate::UserType;
use crate::{DbError, ShopBackend};

use anyhow::{bail, Result};
use function_name::named;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

impl ShopBackend {
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
