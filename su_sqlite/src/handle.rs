use deadpool_sqlite::{CreatePoolError, Pool, PoolConfig};

use crate::config::Config;

pub struct Handle {
    pub write_pool: Pool,
    pub read_pool: Pool,
}

#[derive(Debug)]
pub enum CreateHandleError {
    Pool(CreatePoolError),
}

impl std::error::Error for CreateHandleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for CreateHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateHandleError::Pool(create_pool_error) => write!(f, "could not create DB pool {}", create_pool_error),
        }
    }
}

impl From<CreatePoolError> for CreateHandleError {
    fn from(value: CreatePoolError) -> Self {
        Self::Pool(value)
    }
}

impl Handle {
    pub fn new(config: Config) -> Result<Self, CreateHandleError> {
        let mut read_config = deadpool_sqlite::Config::new(config.name);
        read_config.pool = config.pool_config;

        let mut write_config = read_config.clone();

        // There should only be one active write connection at a time
        let write_pool_config = config
            .pool_config
            .and_then(|cfg| Some(PoolConfig { max_size: 1, ..cfg }));

        write_config.pool = write_pool_config;

        let read_pool = read_config.create_pool(deadpool_sqlite::Runtime::Tokio1)?;
        let write_pool = write_config.create_pool(deadpool_sqlite::Runtime::Tokio1)?;

        Ok(Self {
            read_pool,
            write_pool,
        })
    }

    pub async fn get_read_conn(
        &self,
    ) -> Result<deadpool_sqlite::Connection, deadpool_sqlite::PoolError> {
        let conn = self.read_pool.get().await?;
        Ok(conn)
    }

    pub async fn get_write_conn(
        &self,
    ) -> Result<deadpool_sqlite::Connection, deadpool_sqlite::PoolError> {
        let conn = self.write_pool.get().await?;
        Ok(conn)
    }
}
