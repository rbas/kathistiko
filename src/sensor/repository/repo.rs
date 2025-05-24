use thiserror::Error;

use crate::sensor::model::{LivingRoom, Outdoor};

use super::prometheus::{get_sensors_data, process_sensor_data, RepositoryError};

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Cannot load sensors data due to the following error{0}")]
    CannotLoadData(String),
}

impl From<RepositoryError> for Errors {
    fn from(err: RepositoryError) -> Self {
        Errors::CannotLoadData(format!(": {err}"))
    }
}

pub async fn load_sensors_data(
    hostname: &str,
    username: &str,
    password: &str,
) -> Result<(Option<LivingRoom>, Option<Outdoor>), Errors> {
    let sensors = get_sensors_data(hostname, username, password).await?;
    let (living_room, outdoor) = process_sensor_data(sensors);
    Ok((living_room, outdoor))
}
