use super::temperature::{Errors, SensorData, SensorName};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Battery {
    percentage_sensor: SensorData,
}

impl Battery {
    pub fn percentage(&self) -> u8 {
        ((self.percentage_sensor.value / 5.0).round() * 5.0).clamp(0.0, 100.0) as u8
    }
}

impl TryFrom<Vec<SensorData>> for Battery {
    type Error = Errors;

    fn try_from(sensors: Vec<SensorData>) -> Result<Self, Self::Error> {
        let percentage = extract_sensor_data(sensors, SensorName::KathistikoBatteryPercentage)?;
        Ok(Self {
            percentage_sensor: percentage,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LivingRoom {
    temperature_sensor: SensorData,
    humidity_sensor: SensorData,
}

impl LivingRoom {
    pub fn temperature(&self) -> i16 {
        self.temperature_sensor.value.round() as i16
    }

    pub fn humidity(&self) -> u8 {
        self.humidity_sensor.value.round().clamp(0.0, 100.0) as u8
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
    pub fn temperature(&self) -> i16 {
        self.temperature_sensor.value.round() as i16
    }

    pub fn humidity(&self) -> u8 {
        self.humidity_sensor.value.round().clamp(0.0, 100.0) as u8
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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn battery_with_percentage(value: f32) -> Battery {
        Battery::try_from(vec![SensorData::new(
            SensorName::KathistikoBatteryPercentage,
            value,
            Utc::now(),
        )])
        .unwrap()
    }

    #[test]
    fn battery_percentage_is_rounded_and_bounded() {
        assert_eq!(battery_with_percentage(77.4).percentage(), 75);
        assert_eq!(battery_with_percentage(77.5).percentage(), 80);
        assert_eq!(battery_with_percentage(78.6).percentage(), 80);
        assert_eq!(battery_with_percentage(-5.0).percentage(), 0);
        assert_eq!(battery_with_percentage(105.0).percentage(), 100);
    }

    #[test]
    fn living_room_values_are_rounded_for_display() {
        let room = LivingRoom::try_from(vec![
            SensorData::new(SensorName::KathistikoTemperature, 21.6, Utc::now()),
            SensorData::new(SensorName::KathistikoHumidity, 48.4, Utc::now()),
        ])
        .unwrap();

        assert_eq!(room.temperature(), 22);
        assert_eq!(room.humidity(), 48);
    }

    #[test]
    fn outdoor_values_are_rounded_for_display() {
        let outdoor = Outdoor::try_from(vec![
            SensorData::new(SensorName::KairosTemperature, -2.6, Utc::now()),
            SensorData::new(SensorName::KairosHumidity, 101.0, Utc::now()),
        ])
        .unwrap();

        assert_eq!(outdoor.temperature(), -3);
        assert_eq!(outdoor.humidity(), 100);
    }
}
