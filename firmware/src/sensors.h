#pragma once

#include <Wire.h>
#include "Adafruit_SHT4x.h"

Adafruit_SHT4x sht4 = Adafruit_SHT4x();

struct SensorData {
  float temperature;
  float humidity;
  bool valid;
};

SensorData readSensors() {
  SensorData data{0.0f, 0.0f, false};

  if (!sht4.begin()) {
    Serial.println("SHT4x not found");
    Serial.println("Check the connection");
    return data;
  }

  sht4.setPrecision(SHT4X_HIGH_PRECISION);
  sht4.setHeater(SHT4X_NO_HEATER);

  sensors_event_t humidity, temp;  // temperature and humidity variables

  if (!sht4.getEvent(&humidity, &temp)) {
    Serial.println("Failed to read SHT4x sensor data");
    return data;
  }
  Serial.print("Temperature: ");
  Serial.print(temp.temperature);
  Serial.println(" degC");
  Serial.print("Humidity: ");
  Serial.print(humidity.relative_humidity);
  Serial.println("% rH");

  data.humidity = humidity.relative_humidity;
  data.temperature = temp.temperature;
  data.valid = true;

  return data;
}
