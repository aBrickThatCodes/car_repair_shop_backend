use thiserror::Error;

#[derive(Debug, Error)]
pub struct DbError(String);

impl DbError {
    pub fn new(error: String) -> Self {
        DbError(error)
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
pub struct LoginError(String);

impl LoginError {
    pub fn new(error: String) -> Self {
        LoginError(error)
    }
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
pub struct RegisterClientError;

impl std::fmt::Display for RegisterClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("email already registered")
    }
}

#[derive(Debug, Error)]
pub struct PermissionError;

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("permission denied")
    }
}
