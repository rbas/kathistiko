use chrono::{DateTime, Utc};
use core::fmt;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Missing data from sensor {0}")]
    MissingSensorData(SensorName),
    #[error("Invalid sensor name")]
    InvalidSensorName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensorName {
    KairosTemperature,
    KairosHumidity,
    KathistikoTemperature,
    KathistikoHumidity,
}

impl fmt::Display for SensorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SensorName::KairosTemperature => "Kairos.Temperature",
            SensorName::KairosHumidity => "Kairos.Humidity",
            SensorName::KathistikoTemperature => "Kathistiko.Temperature",
            SensorName::KathistikoHumidity => "Kathistiko.Humidity",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<&str> for SensorName {
    type Error = Errors;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sensor.kairos_temperature" => Ok(SensorName::KairosTemperature),
            "sensor.kairos_humidity" => Ok(SensorName::KairosHumidity),
            "sensor.kathistiko_temperature" => Ok(SensorName::KathistikoTemperature),
            "sensor.kathistiko_humidity" => Ok(SensorName::KathistikoHumidity),
            _ => Err(Errors::InvalidSensorName),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(super) struct SensorData {
    pub(super) name: SensorName,
    pub(super) value: f32,
    pub(super) captured_at: DateTime<Utc>,
}

impl SensorData {
    pub fn new(name: SensorName, value: f32, captured_at: DateTime<Utc>) -> Self {
        Self {
            name,
            value,
            captured_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_valid_sensors() {
        match SensorName::try_from("sensor.kairos_temperature") {
            Ok(name) => assert!(matches!(name, SensorName::KairosTemperature)),
            Err(_) => panic!("Expected Ok(SensorName::KairosTemperature), got Err"),
        }
        match SensorName::try_from("sensor.kairos_humidity") {
            Ok(name) => assert!(matches!(name, SensorName::KairosHumidity)),
            Err(_) => panic!("Expected Ok(SensorName::KairosHumidity), got Err"),
        }

        match SensorName::try_from("sensor.kathistiko_temperature") {
            Ok(name) => assert!(matches!(name, SensorName::KathistikoTemperature)),
            Err(_) => panic!("Expected Ok(SensorName::KathistikoTemperature), got Err"),
        }

        match SensorName::try_from("sensor.kathistiko_humidity") {
            Ok(name) => assert!(matches!(name, SensorName::KathistikoHumidity)),
            Err(_) => panic!("Expected Ok(SensorName::KathistikoHumidity), got Err"),
        }
    }

    #[test]
    fn test_try_from_invalid_sensor() {
        match SensorName::try_from("sensor.unknown_temperature") {
            Ok(name) => panic!("Expected Err, got Ok({:?})", name),
            Err(err) => assert!(matches!(err, Errors::InvalidSensorName)),
        }
    }
}
