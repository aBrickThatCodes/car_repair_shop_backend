use crate::{
    entities::client::{self, Car},
    entities::prelude::*,
    *,
};

use anyhow::{bail, Result};
use function_name::named;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap());

pub static HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\$2[aby]?\$\d{1,2}\$[./A-Za-z0-9]{53}$").unwrap());

impl ShopBackend {
    pub fn login_check(&self, func_name: &str) -> Result<()> {
        if matches!(self.user.user_type(), UserType::NotLoggedIn) {
            func_name.to_string();
        }
        Ok(())
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
                    let mut client_active: client::ActiveModel = client.into();
                    client_active.car = Set(Some(Car {
                        make: make.to_owned(),
                        model: model.to_owned(),
                    }));
                    client_active.update(&self.db).await?;
                    Ok(())
                }
            },
            None => bail!(DbError(format!("client {client_id} does not exist"))),
        }
    }
}
