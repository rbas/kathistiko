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
    #[serde(default)]
    pub display: DisplaySettings,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct DisplaySettings {
    pub enabled: bool,
    pub source_url: String,
    pub refresh_interval_seconds: u64,
    pub container_binary: String,
    pub working_directory: String,
    pub output_directory: String,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            source_url: "http://127.0.0.1:8000/calendar".to_string(),
            refresh_interval_seconds: 900,
            container_binary: "docker".to_string(),
            working_directory: ".".to_string(),
            output_directory: "output".to_string(),
        }
    }
}
impl Settings {
    pub fn new(path: &Path) -> Result<Self, ServiceConfigurationError> {
        let display_path = path.with_file_name("display.local.toml");
        let settings = Config::builder()
            .add_source(File::from(path))
            .add_source(File::from(display_path).required(false))
            .build()?;

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
