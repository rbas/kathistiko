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
    type Error = TemperatureSensorsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sensor.kairos_temperature" => Ok(SensorName::KairosTemperature),
            "sensor.kairos_humidity" => Ok(SensorName::KairosHumidity),
            "sensor.kathistiko_temperature" => Ok(SensorName::KathistikoTemperature),
            "sensor.kathistiko_humidity" => Ok(SensorName::KathistikoHumidity),
            _ => Err(TemperatureSensorsError::InvalidSensorName),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SensorData {
    name: SensorName,
    value: f32,
    captured_at: DateTime<Utc>,
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LivingRoom {
    temperature_sensor: SensorData,
    humidity_sensor: SensorData,
}

impl LivingRoom {
    pub fn temperature(&self) -> f32 {
        self.temperature_sensor.value
    }

    pub fn humidity(&self) -> f32 {
        self.humidity_sensor.value
    }
}

impl TryFrom<Vec<SensorData>> for LivingRoom {
    type Error = Errors;

    fn try_from(sensors: Vec<SensorData>) -> Result<Self, Self::Error> {
        let temperature = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::KathistikoTemperature)
            .ok_or(Errors::MissingSensorData(SensorName::KathistikoTemperature))?;

        let humidity = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::KathistikoHumidity)
            .ok_or(Errors::MissingSensorData(SensorName::KathistikoHumidity))?;

        Ok(Self {
            temperature_sensor: temperature.clone(),
            humidity_sensor: humidity.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Outdoor {
    temperature_sensor: SensorData,
    humidity_sensor: SensorData,
}

impl Outdoor {
    pub fn temperature(&self) -> f32 {
        self.temperature_sensor.value
    }

    pub fn humidity(&self) -> f32 {
        self.humidity_sensor.value
    }
}

impl TryFrom<Vec<SensorData>> for Outdoor {
    type Error = Errors;

    fn try_from(sensors: Vec<SensorData>) -> Result<Self, Self::Error> {
        let temperature = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::KairosTemperature)
            .ok_or(Errors::MissingSensorData(SensorName::KairosTemperature))?;

        let humidity = sensors
            .iter()
            .find(|sensor| sensor.name == SensorName::KairosHumidity)
            .ok_or(Errors::MissingSensorData(SensorName::KairosHumidity))?;

        Ok(Self {
            temperature_sensor: temperature.clone(),
            humidity_sensor: humidity.clone(),
        })
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

impl TryFrom<ResultItem> for SensorData {
    type Error = TemperatureSensorsError;

    fn try_from(item: ResultItem) -> Result<Self, Self::Error> {
        let name = SensorName::try_from(item.metric.entity.as_str())
            .map_err(|_| TemperatureSensorsError::InvalidSensorName)?;
        let value: f32 = item
            .value
            .1
            .parse()
            .map_err(|_| TemperatureSensorsError::InvalidTemperature)?;
        let timestamp = Utc
            .timestamp_opt(item.value.0 as i64, 0)
            .single()
            .ok_or(TemperatureSensorsError::InvalidTimestamp)?;
        Ok(SensorData::new(name, value, timestamp))
    }
}

impl TryFrom<ApiResponse> for Vec<SensorData> {
    type Error = TemperatureSensorsError;

    fn try_from(api_response: ApiResponse) -> Result<Self, Self::Error> {
        api_response
            .data
            .result
            .into_iter()
            .map(SensorData::try_from)
            .collect()
    }
}

pub async fn download_temperature_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<ApiResponse, TemperatureSensorsError> {
    let path = r#"/api/v1/query?query={__name__%3D~"hass_sensor_unit_u0x25u0x20rh|hass_sensor_unit_celsius"%2Centity%3D~"sensor.kathistiko_humidity|sensor.kairos_humidity|sensor.kathistiko_temperature|sensor.kairos_temperature"}"#;
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
) -> Result<Vec<SensorData>, TemperatureSensorsError> {
    let api_response = download_temperature_data(hostname, username, password).await?;
    let sensors = Vec::try_from(api_response)?;
    Ok(sensors)
}

pub fn process_sensor_data(sensors: Vec<SensorData>) -> (Option<LivingRoom>, Option<Outdoor>) {
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
                  1748071727.064,
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
                  1748071727.064,
                  "21.67"
                ]
              },
              {
                "metric": {
                  "__name__": "hass_sensor_unit_u0x25u0x20rh",
                  "domain": "sensor",
                  "entity": "sensor.kairos_humidity",
                  "friendly_name": "Kairos humidity",
                  "instance": "192.168.42.235:8123",
                  "job": "hass"
                },
                "value": [
                  1748071727.064,
                  "54.7"
                ]
              },
              {
                "metric": {
                  "__name__": "hass_sensor_unit_u0x25u0x20rh",
                  "domain": "sensor",
                  "entity": "sensor.kathistiko_humidity",
                  "friendly_name": "Kathistiko humidity",
                  "instance": "192.168.42.235:8123",
                  "job": "hass"
                },
                "value": [
                  1748071727.064,
                  "57.66"
                ]
              }
            ]
          }
        });

        let api_response: ApiResponse = serde_json::from_value(json_data).unwrap();
        let sensors: Vec<SensorData> = api_response.try_into().unwrap();
        for sensor in sensors {
            match sensor.name {
                SensorName::KairosTemperature => {
                    assert_eq!(sensor.value, 16.05);
                    assert_eq!(sensor.captured_at.to_string(), "2025-05-24 07:28:47 UTC");
                }
                SensorName::KairosHumidity => {
                    assert_eq!(sensor.value, 54.7);
                    assert_eq!(sensor.captured_at.to_string(), "2025-05-24 07:28:47 UTC");
                }
                SensorName::KathistikoTemperature => {
                    assert_eq!(sensor.value, 21.67);
                    assert_eq!(sensor.captured_at.to_string(), "2025-05-24 07:28:47 UTC");
                }
                SensorName::KathistikoHumidity => {
                    assert_eq!(sensor.value, 57.66);
                    assert_eq!(sensor.captured_at.to_string(), "2025-05-24 07:28:47 UTC");
                }
            }
        }
    }
}
