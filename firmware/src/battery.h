#pragma once

#include <Arduino.h>

struct BatteryData {
  float voltage;
  uint8_t percentage;
};

// A compact approximation of a single-cell Li-Po discharge curve. Voltage is
// still published separately because percentage derived from voltage is only
// an estimate and varies with temperature, load, and battery age.
inline uint8_t estimateBatteryPercentage(float voltage) {
  static constexpr float voltagePoints[] = {
      3.30f, 3.50f, 3.60f, 3.70f, 3.75f, 3.80f, 3.85f,
      3.90f, 3.95f, 4.00f, 4.05f, 4.10f, 4.15f, 4.20f,
  };
  static constexpr uint8_t percentagePoints[] = {
      0, 5, 10, 20, 30, 40, 50, 60, 70, 80, 85, 90, 95, 100,
  };

  constexpr size_t pointCount = sizeof(voltagePoints) / sizeof(voltagePoints[0]);
  if (voltage <= voltagePoints[0]) {
    return percentagePoints[0];
  }
  if (voltage >= voltagePoints[pointCount - 1]) {
    return percentagePoints[pointCount - 1];
  }

  for (size_t index = 1; index < pointCount; ++index) {
    if (voltage <= voltagePoints[index]) {
      const float interval = voltagePoints[index] - voltagePoints[index - 1];
      const float position = (voltage - voltagePoints[index - 1]) / interval;
      const float percentage = percentagePoints[index - 1] +
                               position * (percentagePoints[index] - percentagePoints[index - 1]);
      return static_cast<uint8_t>(roundf(percentage));
    }
  }

  return 0;
}

inline BatteryData readBattery() {
  constexpr uint8_t sampleCount = 16;
  uint32_t millivoltSum = 0;

  analogSetPinAttenuation(BATTERY_ADC_PIN, ADC_11db);
  for (uint8_t sample = 0; sample < sampleCount; ++sample) {
    millivoltSum += analogReadMilliVolts(BATTERY_ADC_PIN);
    delay(2);
  }

  const float adcVoltage = (millivoltSum / static_cast<float>(sampleCount)) / 1000.0f;
  const float batteryVoltage = adcVoltage * BATTERY_DIVIDER_RATIO;
  return BatteryData{batteryVoltage, estimateBatteryPercentage(batteryVoltage)};
}

