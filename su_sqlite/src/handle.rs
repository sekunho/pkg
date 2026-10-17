use std::path::PathBuf;

use deadpool::managed::QueueMode;
use deadpool_sqlite::{
    BuildError, Hook, HookError, Pool, PoolConfig, Timeouts,
    rusqlite::{self, TransactionBehavior},
};

pub struct Handle {
    read_pool: Pool,
    write_pool: Pool,
}

pub struct HandleBuilder {
    database: PathBuf,
    write_config: PoolConfig,
    read_config: PoolConfig,
    read_hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,
    write_hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,

    // If enabled, it automatically sets the query_only to true for read connections,
    // and sets BEGIN IMMEDIATE; for write connections.
    strict: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CreateHandleError {
    Build(BuildError),
}

impl std::error::Error for CreateHandleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CreateHandleError::Build(err) => Some(err),
        }
    }
}

impl std::fmt::Display for CreateHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateHandleError::Build(_) => write!(f, "could not create DB pool"),
        }
    }
}

impl From<BuildError> for CreateHandleError {
    fn from(err: BuildError) -> Self {
        Self::Build(err)
    }
}

impl HandleBuilder {
    #[must_use]
    pub fn new(database: PathBuf) -> Self {
        let read_config = PoolConfig::default();

        let mut write_config = PoolConfig::default();
        // We should only have one writer
        write_config.max_size = 1;

        Self {
            database,
            read_config,
            write_config,
            read_hook_fn: None,
            write_hook_fn: None,
            strict: true,
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

    pub fn set_read_post_create(
        &mut self,
        hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,
    ) {
        self.read_hook_fn = hook_fn;
    }

    pub fn set_write_post_create(
        &mut self,
        hook_fn: Option<fn(&rusqlite::Connection) -> Result<(), rusqlite::Error>>,
    ) {
        self.write_hook_fn = hook_fn;
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict
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
        let mut read_pool = read_config
            .builder(deadpool_sqlite::Runtime::Tokio1)
            .expect("infallible");

        let mut write_pool = write_config
            .builder(deadpool_sqlite::Runtime::Tokio1)
            .expect("infallible");

        if self.strict {
            read_pool = read_pool.post_create(Hook::async_fn(move |conn, _| {
                Box::pin(async move {
                    conn.interact(|conn| conn.pragma_update(None, "query_only", true))
                        .await
                        .map_err(|e| HookError::message(format!("{e}")))?
                        .map_err(|e| HookError::message(format!("{e}")))
                })
            }));

            write_pool = write_pool.post_create(Hook::async_fn(move |conn, _| {
                Box::pin(async move {
                    conn.interact(|conn| -> Result<(), rusqlite::Error> {
                        conn.set_transaction_behavior(TransactionBehavior::Immediate);
                        Ok(())
                    })
                    .await
                    .map_err(|e| HookError::message(format!("{e}")))?
                    .map_err(|e| HookError::message(format!("{e}")))
                })
            }));
        }

        if let Some(hook_fn) = self.read_hook_fn {
            read_pool = read_pool.post_create(Hook::async_fn(move |conn, _| {
                Box::pin(async move {
                    conn.interact(move |conn| hook_fn(conn))
                        .await
                        .map_err(|e| HookError::message(format!("{e}")))?
                        .map_err(|e| HookError::message(format!("{e}")))
                })
            }));
        }

        if let Some(hook_fn) = self.write_hook_fn {
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
            write_pool,
        })
    }
}

impl Handle {
    #[must_use]
    pub fn builder(database: PathBuf) -> HandleBuilder {
        HandleBuilder::new(database)
    }

    pub async fn get_read_conn(
        &self,
    ) -> Result<deadpool_sqlite::Connection, deadpool_sqlite::PoolError> {
        self.read_pool.get().await
    }

    pub async fn get_write_conn(
        &self,
    ) -> Result<deadpool_sqlite::Connection, deadpool_sqlite::PoolError> {
        self.write_pool.get().await
    }
}
