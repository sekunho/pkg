use core::fmt::Debug;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pool_size: u16,
    pub password: Option<String>,
    pub password_file: Option<PathBuf>,
    pub ca_cert_file: Option<PathBuf>,
    pub client_cert_file: Option<PathBuf>,
    pub client_key_file: Option<PathBuf>,
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            host,
            port,
            user,
            pool_size,
            password,
            password_file,
            ca_cert_file,
            client_cert_file,
            client_key_file,
        } = self;

        let password = if password.is_some() {
            Some("[REDACTED]")
        } else {
            None
        };

        write!(f, "Config {{ name: {:?}, host: {:?}, port: {:?}, user: {:?}, pool_size: {:?}, password: {:?}, password_file: {:?}, ca_cert_file: {:?}, client_cert_file: {:?}, client_key_file: {:?} }}", name, host, port, user, pool_size, password, password_file, ca_cert_file, client_cert_file, client_key_file)
    }
}

impl Config {
    pub fn from_env(prefix: &str, separator: &str) -> Result<Self, config::ConfigError> {
        let source = config::Environment::with_prefix(prefix)
            .try_parsing(true)
            .separator(separator);

        let config = config::Config::builder().add_source(source).build()?;
        let db_config: Config = config.try_deserialize()?;
        Ok(db_config)
    }
}
