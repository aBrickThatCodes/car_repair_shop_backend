use thiserror::Error;

#[derive(Debug, Error)]
pub struct DbError(pub String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
pub enum RegisterClientError {
    EmailAlreadyRegistered(String),
    EmailIncorrectFormat(String),
    PasswordNotHashed,
    AlreadyLoggedIn,
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
            RegisterClientError::PasswordNotHashed => {
                f.write_str("password hash not a bcrypt hash")
            }
            RegisterClientError::AlreadyLoggedIn => {
                f.write_str("cannot register a client if already logged in")
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum LoginError {
    AlreadyLoggedIn,
    EmployeeNotRegistered(i32),
    EmployeeIncorrectPassword(i32),
    EmailNotRegistered(String),
    EmailIncorrectFormat(String),
    ClientIncorrectPassword(String),
    PasswordNotHashed,
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::EmployeeNotRegistered(id) => write!(f, "no employee with ID {id}"),
            LoginError::EmailNotRegistered(email) => {
                write!(f, "no user with email {email}")
            }
            LoginError::EmailIncorrectFormat(email) => {
                write!(f, "{email} is not a correct email address")
            }
            LoginError::PasswordNotHashed => f.write_str("password hash is not a bcrypt hash"),
            LoginError::ClientIncorrectPassword(email) => {
                write!(f, "incorrect password for {email}")
            }
            LoginError::EmployeeIncorrectPassword(id) => {
                write!(f, "incorrect password for employee {id}")
            }
            LoginError::AlreadyLoggedIn => f.write_str("already logged in"),
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
