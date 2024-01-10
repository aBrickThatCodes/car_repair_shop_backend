use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{NotLoggedInError, ShopBackend, UserType};

pub static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap());

pub static HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\$2[aby]?\$\d{1,2}\$[./A-Za-z0-9]{53}$").unwrap());

impl ShopBackend {
    pub fn login_check(&self, func_name: &str) -> Result<()> {
        if matches!(self.user.user_type(), UserType::NotLoggedIn) {
            bail!(NotLoggedInError::new(func_name));
        }
        Ok(())
    }
}
