use deadpool_sqlite::{CreatePoolError, Pool};

use crate::config::Config;

pub struct Handle {
    pub pool: Pool,
}

#[derive(Debug, Error)]
pub enum CreateHandleError {
    #[error("failed to create DB pool. reason: {0}")]
    Pool(#[from] CreatePoolError),
}

impl Handle {
    pub fn new(config: Config) -> Result<Self, CreateHandleError> {
        let mut cfg = deadpool_sqlite::Config::new(config.name);
        let pool = cfg.create_pool(deadpool_sqlite::Runtime::Tokio1)?;
        Ok(Self { pool })
    }
}
