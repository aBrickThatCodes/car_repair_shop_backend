use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("problem reading .env file")]
    Dotenv(#[from] dotenvy::Error),
    #[error("database  error")]
    Database(#[from] DbErr),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("client {0} does not exist")]
    Client(u32),
    #[error("employee {0} does not exist")]
    Employee(u32),
    #[error("order {0} does not exist")]
    Order(u32),
    #[error("report {0} does not exist")]
    Report(u32),
    #[error("permission denied")]
    Permission,
    #[error("{0}")]
    NotLoggedIn(#[from] NotLoggedInError),
    #[error("database error: {0}")]
    Database(#[from] DbErr),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum RegisterClientError {
    #[error("already logged in")]
    AlreadyLoggedIn,
    #[error("email {0} already registered")]
    EmailAlreadyRegistered(String),
    #[error("{0} is not a correct email address")]
    EmailIncorrectFormat(String),
    #[error("password hash is not a bcrypt hash")]
    PasswordNotHashed,
    #[error("database error: {0}")]
    Database(#[from] DbErr),
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("already logged in")]
    AlreadyLoggedIn,
    #[error("no employee with ID {0}")]
    EmployeeNotRegistered(u32),
    #[error("incorrect password for employee {0}")]
    EmployeeIncorrectPassword(u32),
    #[error("no user with email {0}")]
    EmailNotRegistered(String),
    #[error("{0} is not a correct email address")]
    EmailIncorrectFormat(String),
    #[error("incorrect password for {0}")]
    ClientIncorrectPassword(String),
    #[error("password hash is not a bcrypt hash")]
    PasswordNotHashed,
    #[error("database error: {0}")]
    Database(#[from] DbErr),
}

#[derive(Debug, Error)]
#[error("function {0} requires being logged in")]
pub struct NotLoggedInError(pub String);
