#pragma once

// Copy this file to conf.h and provide device-specific values. conf.h is
// ignored by Git and must never be committed.
#define WIFI_SSID "replace-me"
#define WIFI_PASSWORD "replace-me"

#define HOSTNAME "kathistiko"
#define OTA_PASSWORD "replace-me"

#define MQTT_USERNAME "replace-me"
#define MQTT_PASSWORD "replace-me"
#define MQTT_SERVER "192.0.2.1"
#define MQTT_SERVER_PORT 1883

// Keep the public endpoint during the firmware migration. This can later be
// changed in conf.h to an internal address without rebuilding source history.
#define IMAGE_SERVER_URL "https://kathistiko-backend.masovice.net/latest"

#define SLEEP_SECONDS 1800

// LaskaKit ESPink v2.x battery voltage divider.
#define BATTERY_ADC_PIN 34
#define BATTERY_DIVIDER_RATIO 1.769388f
