use std::{path::PathBuf, sync::Mutex};

use deadpool::managed::QueueMode;
use deadpool_sqlite::{CreatePoolError, Pool, PoolConfig, Timeouts};

pub struct Handle {
    pub write_pool: Mutex<Pool>,
    pub read_pool: Pool,
}

#[derive(Debug, Clone)]
pub struct HandleBuilder {
    database: PathBuf,
    write_config: PoolConfig,
    read_config: PoolConfig,
}

#[derive(Debug)]
pub enum CreateHandleError {
    Pool(CreatePoolError),
}

pub struct WriteObject();

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
            CreateHandleError::Pool(create_pool_error) => {
                write!(f, "could not create DB pool {}", create_pool_error)
            }
        }
    }
}

impl From<CreatePoolError> for CreateHandleError {
    fn from(value: CreatePoolError) -> Self {
        Self::Pool(value)
    }
}

impl HandleBuilder {
    pub fn new(database: PathBuf) -> Self {
        let read_config = PoolConfig::default();

        let mut write_config = PoolConfig::default();
        // We should only have one writer
        write_config.max_size = 1;

        Self{ database, read_config, write_config }
    }

    pub fn set_read_pool_max_size(mut self, pool_max_size: usize) {
        self.read_config.max_size = pool_max_size;
    }

    pub fn set_read_pool_timeouts(mut self, timeouts: Timeouts) {
        self.read_config.timeouts = timeouts;
    }

    pub fn set_write_pool_timeouts(mut self, timeouts: Timeouts) {
        self.write_config.timeouts = timeouts;
    }

    pub fn set_read_pool_queue_mode(mut self, queue_mode: QueueMode) {
        self.read_config.queue_mode = queue_mode;
    }

    pub fn set_write_pool_queue_mode(mut self, queue_mode: QueueMode) {
        self.write_config.queue_mode = queue_mode;
    }

    pub fn build(self) -> Result<Handle, CreatePoolError> {
        let read_config = deadpool_sqlite::Config{path: self.database.clone(), pool: Some(self.read_config)};
        let write_config = deadpool_sqlite::Config{path: self.database, pool: Some(self.write_config)};
        let read_pool = read_config.create_pool(deadpool_sqlite::Runtime::Tokio1)?;
        let write_pool = write_config.create_pool(deadpool_sqlite::Runtime::Tokio1)?;

        Ok(Handle{ read_pool, write_pool: Mutex::new(write_pool) })
    }
}

impl Handle {
    pub fn builder(database: PathBuf) -> HandleBuilder {
        HandleBuilder::new(database)
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
        let conn = self.read_pool.get().await?;
        Ok(conn)
    }
}
