use std::fmt::Display;

use deadpool_sqlite::{InteractError, PoolError};

#[derive(Debug)]
#[non_exhaustive]
pub enum DbError {
    Db(rusqlite::Error),
    Pool(PoolError),
    Interact(InteractError),
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Pool(_err) => write!(f, "failed to acquire pool"),
            DbError::Interact(_err) => write!(f, "transaction failure"),
            DbError::Db(_err) => write!(f, "query failure"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Pool(err) => Some(err),
            DbError::Interact(err) => Some(err),
            DbError::Db(err) => Some(err),
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Db(value)
    }
}

impl From<PoolError> for DbError {
    fn from(value: PoolError) -> Self {
        Self::Pool(value)
    }
}

impl From<InteractError> for DbError {
    fn from(value: InteractError) -> Self {
        Self::Interact(value)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CreateUserError {
    Hash(argon2::password_hash::Error),
    Db(rusqlite::Error),
}

impl Display for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateUserError::Hash(_err) => write!(f, "failed to hash password"),
            CreateUserError::Db(_err) => write!(f, "query failure"),
        }
    }
}

impl std::error::Error for CreateUserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CreateUserError::Hash(err) => Some(err),
            CreateUserError::Db(err) => Some(err),
        }
    }
}

impl From<argon2::password_hash::Error> for CreateUserError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Hash(value)
    }
}

impl From<rusqlite::Error> for CreateUserError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Db(value)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum VerifyError {
    Join(tokio::task::JoinError),
    Verify(argon2::password_hash::Error),
}

impl Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::Join(_err) => write!(f, "password verification task failed"),
            VerifyError::Verify(_err) => write!(f, "failed to verify password"),
        }
    }
}

impl std::error::Error for VerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VerifyError::Join(err) => Some(err),
            VerifyError::Verify(err) => Some(err),
        }
    }
}

impl From<tokio::task::JoinError> for VerifyError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value)
    }
}

impl From<argon2::password_hash::Error> for VerifyError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Verify(value)
    }
}
