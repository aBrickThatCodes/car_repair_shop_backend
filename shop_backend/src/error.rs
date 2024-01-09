use thiserror::Error;

#[derive(Debug, Error)]
pub struct DbError(pub String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
pub struct LoginError(pub String);

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
pub enum RegisterClientError {
    EmailAlreadyRegistered(String),
    EmailIncorrectFormat(String),
}

impl std::fmt::Display for RegisterClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterClientError::EmailAlreadyRegistered(email) => {
                write!(f, "email {email} already registered")
            }
            RegisterClientError::EmailIncorrectFormat(email) => {
                write!(f, "{email} is not a correct email address")
            }
        }
    }
}

#[derive(Debug, Error)]
pub struct PermissionError;

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("permission denied")
    }
}

#[derive(Debug, Error)]
pub struct NotLoggedInError(pub String);

impl std::fmt::Display for NotLoggedInError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} requires being logged in", self.0)
    }
}
