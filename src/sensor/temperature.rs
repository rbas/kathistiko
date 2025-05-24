use chrono::{DateTime, TimeZone, Utc};
use core::fmt;
use reqwest::Error as ReqwestError;
use serde::Deserialize;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemperatureSensorsError {
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    #[error("Invalid response format")]
    InvalidResponseFormat,
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Invalid sensor name")]
    InvalidSensorName,
    #[error("Invalid temperature value")]
    InvalidTemperature,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

#[derive(Debug, Error)]
pub enum Errors {
    #[error("No sensor data found for {0}")]
    NoSensorDataFound(String),
    #[error("Missing data from sensor {0}")]
    MissingSensorData(SensorName),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SensorName {
    Kairos,
    Kathistiko,
}

impl fmt::Display for SensorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SensorName::Kairos => "Kairos",
            SensorName::Kathistiko => "Kathistiko",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<&str> for SensorName {
    type Error = TemperatureSensorsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sensor.kairos_temperature" => Ok(SensorName::Kairos),
            "sensor.kathistiko_temperature" => Ok(SensorName::Kathistiko),
            _ => Err(TemperatureSensorsError::InvalidSensorName),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Sensor {
    pub name: SensorName,
    pub temperature: f32,
    pub timestamp: DateTime<Utc>,
}

impl Sensor {
    pub fn new(name: SensorName, temperature: f32, timestamp: DateTime<Utc>) -> Self {
        Self {
            name,
            temperature,
            timestamp,
        }
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LivingRoom {
    pub temperature: f32,
}

impl LivingRoom {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}

impl TryFrom<Vec<Sensor>> for LivingRoom {
    type Error = Errors;

    fn try_from(sensors: Vec<Sensor>) -> Result<Self, Self::Error> {
        let temperature = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::Kathistiko)
            .ok_or(Errors::MissingSensorData(SensorName::Kathistiko))?
            .temperature;

        Ok(Self { temperature })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Outdoor {
    pub temperature: f32,
}

impl Outdoor {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}

impl TryFrom<Vec<Sensor>> for Outdoor {
    type Error = Errors;

    fn try_from(sensors: Vec<Sensor>) -> Result<Self, Self::Error> {
        let temperature = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::Kairos)
            .ok_or(Errors::MissingSensorData(SensorName::Kairos))?
            .temperature;

        Ok(Self { temperature })
    }
}

#[derive(Debug, Deserialize)]
pub struct Metric {
    pub entity: String,
}

#[derive(Debug, Deserialize)]
pub struct ResultItem {
    pub metric: Metric,
    pub value: (f64, String),
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub result: Vec<ResultItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub data: Data,
}

impl TryFrom<ResultItem> for Sensor {
    type Error = TemperatureSensorsError;

    fn try_from(item: ResultItem) -> Result<Self, Self::Error> {
        let name = SensorName::try_from(item.metric.entity.as_str())
            .map_err(|_| TemperatureSensorsError::InvalidSensorName)?;
        let temperature: f32 = item
            .value
            .1
            .parse()
            .map_err(|_| TemperatureSensorsError::InvalidTemperature)?;
        let timestamp = Utc
            .timestamp_opt(item.value.0 as i64, 0)
            .single()
            .ok_or(TemperatureSensorsError::InvalidTimestamp)?;
        Ok(Sensor::new(name, temperature, timestamp))
    }
}

impl TryFrom<ApiResponse> for Vec<Sensor> {
    type Error = TemperatureSensorsError;

    fn try_from(api_response: ApiResponse) -> Result<Self, Self::Error> {
        api_response
            .data
            .result
            .into_iter()
            .map(Sensor::try_from)
            .collect()
    }
}

pub async fn download_temperature_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<ApiResponse, TemperatureSensorsError> {
    let path = "/api/v1/query?query=hass_sensor_unit_celsius{entity%3D~\"sensor.kairos_temperature|sensor.kathistiko_temperature\"}";
    let url = format!("{}{}", hostname, path);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .basic_auth(username, Some(password))
        .send()
        .await
        .map_err(TemperatureSensorsError::NetworkError)?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(TemperatureSensorsError::AuthenticationFailed);
    }

    let api_response = response
        .json::<ApiResponse>()
        .await
        .map_err(|_| TemperatureSensorsError::InvalidResponseFormat)?;

    Ok(api_response)
}

pub async fn get_sensors_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Sensor>, TemperatureSensorsError> {
    let api_response = download_temperature_data(hostname, username, password).await?;
    let sensors = Vec::try_from(api_response)?;
    Ok(sensors)
}

pub fn process_sensor_data(sensors: Vec<Sensor>) -> (Option<LivingRoom>, Option<Outdoor>) {
    let living_room = match LivingRoom::try_from(sensors.clone()) {
        Ok(living_room) => Some(living_room),
        Err(err) => {
            log::error!(
                "Cannot read data from living room temperature sensor {:#?}",
                err
            );
            None
        }
    };

    let outdoor = match Outdoor::try_from(sensors.clone()) {
        Ok(outdoor) => Some(outdoor),
        Err(err) => {
            log::error!(
                "Cannot read data from outdoor temperature sensor {:#?}",
                err
            );
            None
        }
    };

    (living_room, outdoor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_try_from_valid_sensors() {
        match SensorName::try_from("sensor.kairos_temperature") {
            Ok(name) => assert!(matches!(name, SensorName::Kairos)),
            Err(_) => panic!("Expected Ok(SensorName::Kairos), got Err"),
        }

        match SensorName::try_from("sensor.kathistiko_temperature") {
            Ok(name) => assert!(matches!(name, SensorName::Kathistiko)),
            Err(_) => panic!("Expected Ok(SensorName::Kathistiko), got Err"),
        }
    }

    #[test]
    fn test_try_from_invalid_sensor() {
        match SensorName::try_from("sensor.unknown_temperature") {
            Ok(name) => panic!("Expected Err, got Ok({:?})", name),
            Err(err) => assert!(matches!(err, TemperatureSensorsError::InvalidSensorName)),
        }
    }

    #[test]
    fn test_sensor_mapping() {
        let json_data = json!({
          "status": "success",
          "data": {
            "resultType": "vector",
            "result": [
              {
                "metric": {
                  "__name__": "hass_sensor_unit_celsius",
                  "domain": "sensor",
                  "entity": "sensor.kairos_temperature",
                  "friendly_name": "Kairos temperature",
                  "instance": "192.168.42.235:8123",
                  "job": "hass"
                },
                "value": [
                  1747989569.610,
                  "16.05"
                ]
              },
              {
                "metric": {
                  "__name__": "hass_sensor_unit_celsius",
                  "domain": "sensor",
                  "entity": "sensor.kathistiko_temperature",
                  "friendly_name": "Kathistiko temperature",
                  "instance": "192.168.42.235:8123",
                  "job": "hass"
                },
                "value": [
                  1747989569.610,
                  "21.67"
                ]
              }
            ]
          }
        });

        let api_response: ApiResponse = serde_json::from_value(json_data).unwrap();
        let sensors: Vec<Sensor> = api_response.try_into().unwrap();
        for sensor in sensors {
            match sensor.name {
                SensorName::Kairos => {
                    assert_eq!(sensor.temperature, 16.05);
                    assert_eq!(sensor.timestamp.to_string(), "2025-05-23 08:39:29 UTC");
                }
                SensorName::Kathistiko => {
                    assert_eq!(sensor.temperature, 21.67);
                    assert_eq!(sensor.timestamp.to_string(), "2025-05-23 08:39:29 UTC");
                }
            }
        }
    }
}
