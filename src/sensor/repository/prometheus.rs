use chrono::{TimeZone, Utc};

use reqwest::{Error as ReqwestError, StatusCode};
use serde::Deserialize;
use thiserror::Error;

use crate::sensor::{
    model::{Battery, LivingRoom, Outdoor},
    temperature::{Errors, SensorData, SensorName},
};

#[derive(Debug, Error)]
pub(super) enum RepositoryError {
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    #[error("Invalid response format")]
    InvalidResponseFormat(#[source] serde_json::Error),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Prometheus returned HTTP {status}: {body}")]
    UnexpectedStatus { status: StatusCode, body: String },

    #[error("Cannot create sensor due to following error {0}")]
    InvalidSensor(Errors),
    #[error("Invalid sensor value `{0}`")]
    InvalidSensorValue(String),
    #[error("Invalid sensor timestamp value `{0}`")]
    InvalidSensorTimestampValue(String),
}

#[derive(Debug, Deserialize)]
struct Metric {
    entity: String,
}

#[derive(Debug, Deserialize)]
struct ResultItem {
    metric: Metric,
    value: (f64, String),
}

#[derive(Debug, Deserialize)]
struct Data {
    result: Vec<ResultItem>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Data,
}

impl TryFrom<ResultItem> for SensorData {
    type Error = RepositoryError;

    fn try_from(item: ResultItem) -> Result<Self, Self::Error> {
        let name = match SensorName::try_from(item.metric.entity.as_str()) {
            Ok(name) => name,
            Err(error) => return Err(RepositoryError::InvalidSensor(error)),
        };
        let value: f32 = match item.value.1.parse() {
            Ok(val) => val,
            Err(_) => return Err(RepositoryError::InvalidSensorValue(item.value.1.clone())),
        };
        let timestamp = match Utc.timestamp_opt(item.value.0 as i64, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => {
                return Err(RepositoryError::InvalidSensorTimestampValue(
                    item.value.0.to_string(),
                ))
            }
        };
        Ok(SensorData::new(name, value, timestamp))
    }
}

impl TryFrom<ApiResponse> for Vec<SensorData> {
    type Error = RepositoryError;

    fn try_from(api_response: ApiResponse) -> Result<Self, Self::Error> {
        api_response
            .data
            .result
            .into_iter()
            .map(SensorData::try_from)
            .collect()
    }
}

async fn download_sensor_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<ApiResponse, RepositoryError> {
    let path = r#"/api/v1/query?query={__name__%3D~"hass_sensor_unit_u0x25u0x20rh|hass_sensor_unit_celsius|hass_sensor_unit_u0x25|hass_sensor_battery_percent"%2Centity%3D~"sensor.kathistiko_humidity|sensor.kairos_humidity|sensor.kathistiko_temperature|sensor.kairos_temperature|sensor.kathistiko_display_battery_percentage"}"#;
    let url = format!("{}{}", hostname, path);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(RepositoryError::NetworkError)?;
    let response = client
        .get(url)
        .basic_auth(username, Some(password))
        .send()
        .await
        .map_err(RepositoryError::NetworkError)?;

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(RepositoryError::AuthenticationFailed);
    }

    let body = response
        .text()
        .await
        .map_err(RepositoryError::NetworkError)?;

    if !status.is_success() {
        return Err(RepositoryError::UnexpectedStatus {
            status,
            body: body.chars().take(200).collect(),
        });
    }

    let api_response = serde_json::from_str::<ApiResponse>(&body)
        .map_err(RepositoryError::InvalidResponseFormat)?;

    Ok(api_response)
}

pub(super) async fn get_sensors_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<Vec<SensorData>, RepositoryError> {
    let api_response = download_sensor_data(hostname, username, password).await?;
    let sensors = Vec::try_from(api_response)?;
    Ok(sensors)
}

pub(super) fn process_sensor_data(
    sensors: Vec<SensorData>,
) -> (Option<LivingRoom>, Option<Outdoor>, Option<Battery>) {
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

    let battery = match Battery::try_from(sensors) {
        Ok(battery) => Some(battery),
        Err(err) => {
            log::debug!("Battery percentage is not available yet: {:#?}", err);
            None
        }
    };

    (living_room, outdoor, battery)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
              },
              {
                "metric": {
                  "__name__": "hass_sensor_battery_percent",
                  "entity": "sensor.kathistiko_display_battery_percentage"
                },
                "value": [
                  1748071727.064,
                  "78"
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
                SensorName::KathistikoBatteryPercentage => {
                    assert_eq!(sensor.value, 78.0);
                }
            }
        }
    }
}
