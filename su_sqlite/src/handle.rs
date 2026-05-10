use std::{path::PathBuf, sync::Mutex};

use deadpool::managed::QueueMode;
use deadpool_sqlite::{
    rusqlite, BuildError, ConfigError, CreatePoolError, Hook, HookError, Pool, PoolConfig, Timeouts,
};

pub struct Handle {
    pub write_pool: Mutex<Pool>,
    pub read_pool: Pool,
}

pub struct HandleBuilder {
    database: PathBuf,
    write_config: PoolConfig,
    read_config: PoolConfig,
    hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,
}

#[derive(Debug)]
pub enum CreateHandleError {
    Pool(CreatePoolError),
    Config(ConfigError),
    Build(BuildError),
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
            CreateHandleError::Pool(create_pool_error) => {
                write!(f, "could not create DB pool {}", create_pool_error)
            }
            CreateHandleError::Config(err) => write!(f, "could not create DB pool {}", err),
            CreateHandleError::Build(err) => write!(f, "could not create DB pool {}", err),
        }
    }
}

impl From<CreatePoolError> for CreateHandleError {
    fn from(err: CreatePoolError) -> Self {
        Self::Pool(err)
    }
}

impl From<ConfigError> for CreateHandleError {
    fn from(err: ConfigError) -> Self {
        Self::Config(err)
    }
}

impl From<BuildError> for CreateHandleError {
    fn from(err: BuildError) -> Self {
        Self::Build(err)
    }
}

impl HandleBuilder {
    pub fn new(database: PathBuf) -> Self {
        let read_config = PoolConfig::default();

        let mut write_config = PoolConfig::default();
        // We should only have one writer
        write_config.max_size = 1;

        Self {
            database,
            read_config,
            write_config,
            hook_fn: None,
        }
    }

    pub fn set_read_pool_max_size(&mut self, pool_max_size: usize) {
        self.read_config.max_size = pool_max_size;
    }

    pub fn set_read_pool_timeouts(&mut self, timeouts: Timeouts) {
        self.read_config.timeouts = timeouts;
    }

    pub fn set_write_pool_timeouts(&mut self, timeouts: Timeouts) {
        self.write_config.timeouts = timeouts;
    }

    pub fn set_read_pool_queue_mode(&mut self, queue_mode: QueueMode) {
        self.read_config.queue_mode = queue_mode;
    }

    pub fn set_write_pool_queue_mode(&mut self, queue_mode: QueueMode) {
        self.write_config.queue_mode = queue_mode;
    }

    pub fn set_post_create(
        &mut self,
        hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,
    ) {
        self.hook_fn = hook_fn
    }

    pub fn build(self) -> Result<Handle, CreateHandleError> {
        let read_config = deadpool_sqlite::Config {
            path: self.database.clone(),
            pool: Some(self.read_config),
        };
        let write_config = deadpool_sqlite::Config {
            path: self.database,
            pool: Some(self.write_config),
        };
        let mut read_pool = read_config.builder(deadpool_sqlite::Runtime::Tokio1)?;
        let mut write_pool = write_config.builder(deadpool_sqlite::Runtime::Tokio1)?;

        if let Some(hook_fn) = self.hook_fn {
            read_pool = read_pool.post_create(Hook::async_fn(move |conn, _| {
                Box::pin(async move {
                    conn.interact(move |conn| hook_fn(conn))
                        .await
                        .map_err(|e| HookError::message(format!("{e}")))?
                        .map_err(|e| HookError::message(format!("{e}")))
                })
            }));

            write_pool = write_pool.post_create(Hook::async_fn(move |conn, _| {
                Box::pin(async move {
                    conn.interact(move |conn| hook_fn(conn))
                        .await
                        .map_err(|e| HookError::message(format!("{e}")))?
                        .map_err(|e| HookError::message(format!("{e}")))
                })
            }));
        }

        let read_pool = read_pool.build()?;
        let write_pool = write_pool.build()?;

        Ok(Handle {
            read_pool,
            write_pool: Mutex::new(write_pool),
        })
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
