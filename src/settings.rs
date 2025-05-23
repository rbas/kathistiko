use core::fmt;
use std::{error::Error, path::Path, str::FromStr};

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct Settings {
    pub calendars_urls: Vec<String>,
    pub calendar_offset_days: u64,
    pub server_listen_at: String,
    pub tmp_folder: String,
    pub public_folder: String,
    pub cache_ttl_seconds: u64,
    pub prometheus_url: String,
    pub prometheus_username: String,
    pub prometheus_password: String,
}
impl Settings {
    pub fn new(path: &Path) -> Result<Self, ServiceConfigurationError> {
        let settings = Config::builder().add_source(File::from(path)).build()?;

        Self::build(settings)
    }

    pub(super) fn build(settings: Config) -> Result<Self, ServiceConfigurationError> {
        let result: Result<Self, ConfigError> = settings.try_deserialize();
        match result {
            Ok(config) => Ok(config),
            Err(err) => Err(ServiceConfigurationError::from(err)),
        }
    }
}

impl FromStr for Settings {
    type Err = ServiceConfigurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let settings = Config::builder()
            .add_source(File::from_str(s, FileFormat::Toml))
            .build()?;

        Self::build(settings)
    }
}

#[derive(Debug)]
pub enum ServiceConfigurationError {
    ErrorInConfiguration(String),
}
impl fmt::Display for ServiceConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceConfigurationError::ErrorInConfiguration(s) => {
                write!(f, "Configuration error {}", s)
            }
        }
    }
}
impl Error for ServiceConfigurationError {}

impl From<ConfigError> for ServiceConfigurationError {
    fn from(error: ConfigError) -> Self {
        Self::ErrorInConfiguration(error.to_string())
    }
}
