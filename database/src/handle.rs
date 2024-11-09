use core::fmt::Debug;

use deadpool_postgres::Runtime;
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use postgres_openssl::MakeTlsConnector;
use thiserror::Error;
use tokio_postgres::NoTls;

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct Handle {
    pool: deadpool_postgres::Pool,
    pub connector: Option<Connector>,
    pub config: deadpool_postgres::Config,
}

#[derive(Clone)]
pub struct Connector(pub MakeTlsConnector);

impl Debug for Connector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[MakeTlsConnector]")
    }
}

impl From<MakeTlsConnector> for Connector {
    fn from(connector: MakeTlsConnector) -> Self {
        Connector(connector)
    }
}

#[derive(Debug, Error)]
pub enum CreateHandleError {
    #[error("failed to create DB pool")]
    Pool(#[from] deadpool_postgres::CreatePoolError),
    #[error("failed to read password file. reason: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to setup ssl {0}")]
    Ssl(#[from] openssl::error::ErrorStack),
}

impl Handle {
    pub fn new(config: Config) -> Result<Self, CreateHandleError> {
        let mut cfg = deadpool_postgres::Config::new();
        let async_runtime = Some(Runtime::Tokio1);

        let password = match config {
            Config {
                password: Some(_password),
                password_file: Some(path),
                ..
            } => Some(std::fs::read_to_string(path)?.trim().to_string()),
            Config {
                password: Some(password),
                password_file: None,
                ..
            } => Some(password),
            Config {
                password: None,
                password_file: Some(path),
                ..
            } => Some(std::fs::read_to_string(path)?.trim().to_string()),
            _ => None,
        };

        cfg.dbname = Some(config.name);
        cfg.user = Some(config.user);
        cfg.host = Some(config.host);
        cfg.password = password;
        cfg.port = Some(config.port);

        cfg.manager = Some(deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        });

        let handle = match config {
            Config {
                ca_cert_file: Some(server_ca_path),
                client_cert_file: Some(client_cert_path),
                client_key_file: Some(client_key_path),
                ..
            } => {
                let mut builder = SslConnector::builder(SslMethod::tls())?;

                builder.set_ca_file(server_ca_path)?;
                builder.set_certificate_file(client_cert_path, SslFiletype::PEM)?;
                builder.set_private_key_file(client_key_path, SslFiletype::PEM)?;

                let mut connector = MakeTlsConnector::new(builder.build());

                // https://github.com/sfackler/rust-openssl/issues/1572
                connector.set_callback(|connect_config, _domain| {
                    connect_config.set_verify_hostname(false);

                    Ok(())
                });

                Handle {
                    connector: Some(Connector::from(connector.clone())),
                    pool: cfg.create_pool(async_runtime, connector)?,
                    config: cfg,
                }
            }
            _ => Handle {
                pool: cfg.create_pool(async_runtime, NoTls)?,
                connector: None,
                config: cfg,
            },
        };

        Ok(handle)
    }

    pub async fn get_client(
        &self,
    ) -> Result<deadpool_postgres::Client, deadpool_postgres::PoolError> {
        self.pool.get().await
    }
}

impl Config {
    pub fn from_env(prefix: &str, separator: &str) -> Result<Self, config::ConfigError> {
        let source = config::Environment::with_prefix(prefix)
            .try_parsing(true)
            .separator(separator);

        let config = config::Config::builder().add_source(source).build()?;
        let db_config: Config = config.try_deserialize()?;

        tracing::info!("{:?}", db_config);
        Ok(db_config)
    }
}
