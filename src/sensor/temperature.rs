use chrono::{DateTime, TimeZone, Utc};
use reqwest::Error as ReqwestError;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemperatureSensorsError {
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    #[error("Invalid response format")]
    InvalidResponseFormat,
    #[error("Authentication failed")]
    AuthenticationFailed,
}

#[derive(Debug, PartialEq)]
pub enum EntityName {
    Kairos,
    Kathistiko,
}

impl TryFrom<&str> for EntityName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sensor.kairos_temperature" => Ok(EntityName::Kairos),
            "sensor.kathistiko_temperature" => Ok(EntityName::Kathistiko),
            _ => Err("Invalid entity"),
        }
    }
}

#[derive(Debug)]
pub struct Entity {
    pub name: EntityName,
    pub temperature: f32,
    pub timestamp: DateTime<Utc>,
}

impl Entity {
    pub fn new(name: EntityName, temperature: f32, timestamp: DateTime<Utc>) -> Self {
        Self {
            name,
            temperature,
            timestamp,
        }
    }
}

#[derive(Deserialize)]
pub struct Metric {
    pub entity: String,
}

#[derive(Deserialize)]
pub struct ResultItem {
    pub metric: Metric,
    pub value: (f64, String),
}

#[derive(Deserialize)]
pub struct Data {
    pub result: Vec<ResultItem>,
}

#[derive(Deserialize)]
pub struct ApiResponse {
    pub data: Data,
}

impl TryFrom<ResultItem> for Entity {
    type Error = &'static str;

    fn try_from(item: ResultItem) -> Result<Self, Self::Error> {
        let name = EntityName::try_from(item.metric.entity.as_str())?;
        let temperature: f32 = item.value.1.parse().map_err(|_| "Invalid temperature")?;
        let result = Utc.timestamp_opt(item.value.0 as i64, 0).single();
        match result {
            None => Err("Invalid timestamp"),
            Some(timestamp) => Ok(Entity::new(name, temperature, timestamp)),
        }
    }
}

impl TryFrom<ApiResponse> for Vec<Entity> {
    type Error = &'static str;

    fn try_from(api_response: ApiResponse) -> Result<Self, Self::Error> {
        api_response
            .data
            .result
            .into_iter()
            .map(Entity::try_from)
            .collect()
    }
}

pub async fn download_temperature_data(
    url: &str,
    username: &str,
    password: &str,
) -> Result<ApiResponse, TemperatureSensorsError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_try_from_valid_entities() {
        assert_eq!(
            EntityName::try_from("sensor.kairos_temperature"),
            Ok(EntityName::Kairos)
        );
        assert_eq!(
            EntityName::try_from("sensor.kathistiko_temperature"),
            Ok(EntityName::Kathistiko)
        );
    }

    #[test]
    fn test_try_from_invalid_entity() {
        assert_eq!(
            EntityName::try_from("sensor.unknown_temperature"),
            Err("Invalid entity")
        );
    }

    #[test]
    fn test_entity_mapping() {
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
        let entities: Vec<Entity> = api_response.try_into().unwrap();
        for entity in entities {
            match entity.name {
                EntityName::Kairos => {
                    assert_eq!(entity.temperature, 16.05);
                    assert_eq!(entity.timestamp.to_string(), "2025-05-23 08:39:29 UTC");
                }
                EntityName::Kathistiko => {
                    assert_eq!(entity.temperature, 21.67);
                    assert_eq!(entity.timestamp.to_string(), "2025-05-23 08:39:29 UTC");
                }
            }
        }
    }
}
