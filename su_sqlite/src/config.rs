use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub name: String,
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
