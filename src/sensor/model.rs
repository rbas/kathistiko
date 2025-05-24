use super::temperature::{Errors, SensorData, SensorName};

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
        let temperature = extract_sensor_data(sensors.clone(), SensorName::KathistikoTemperature)?;
        let humidity = extract_sensor_data(sensors.clone(), SensorName::KathistikoHumidity)?;

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
        let temperature = extract_sensor_data(sensors.clone(), SensorName::KairosTemperature)?;
        let humidity = extract_sensor_data(sensors.clone(), SensorName::KairosHumidity)?;

        Ok(Self {
            temperature_sensor: temperature.clone(),
            humidity_sensor: humidity.clone(),
        })
    }
}

fn extract_sensor_data(sensors: Vec<SensorData>, name: SensorName) -> Result<SensorData, Errors> {
    sensors
        .into_iter()
        .find(|sensor| sensor.name == name)
        .ok_or(Errors::MissingSensorData(name))
}
