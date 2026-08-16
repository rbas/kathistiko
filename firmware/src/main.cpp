/* Display  Good Display GDEY075T7
 *
 * Board:   LaskaKit ESPink ESP32 e-Paper   https://www.laskakit.cz/laskakit-espink-esp32-e-paper-pcb-antenna/
 * Display: Good Display GDEY075T7          https://www.laskakit.cz/good-display-gdey075t7-7-5--800x480-epaper-displej-grayscale/
 */
#include "Arduino.h"
#include "conf.h"
#include <SPI.h>
#include <WiFi.h>
#include <HTTPClient.h>
#include <Preferences.h>
#include <new>
#include "Display_EPD_W21_spi.h"
#include "Display_EPD_W21.h"
#include "ota.h"
#include <MQTT.h>

#include "sensors.h"
#include "battery.h"

static_assert(POWER == 2, "Unexpected POWER pin");
static_assert(BUSY == 4, "Unexpected BUSY pin");
static_assert(RST == 16, "Unexpected RST pin");
static_assert(DC == 17, "Unexpected DC pin");
static_assert(CS == 5, "Unexpected CS pin");
static_assert(SCK == 18, "Unexpected SCK pin");
static_assert(MISO == -1, "Unexpected MISO pin");
static_assert(MOSI == 23, "Unexpected MOSI pin");

IPAddress getIp();
int8_t getWifiStrength();

enum class ImageFetchResult {
  Updated,
  NotModified,
  Failed,
};

ImageFetchResult fetchData(const String &url, const String &currentEtag,
                           uint8_t *&data, size_t &dataSize, String &responseEtag);

// MQTT path's
const String MQTT_TOPIC_BASE_PATH = "/home/" + String(HOSTNAME);
const String MQTT_TOPIC_OTA_CONFIG_PATH = MQTT_TOPIC_BASE_PATH + "/config/ota";
const String MQTT_TOPIC_BATTERY_VOLTAGE_PATH = MQTT_TOPIC_BASE_PATH + "/battery/voltage";
const String MQTT_TOPIC_BATTERY_PERCENTAGE_PATH = MQTT_TOPIC_BASE_PATH + "/battery/percentage";
const String MQTT_TOPIC_WIFI_STRENGTH_PATH = MQTT_TOPIC_BASE_PATH + "/wifi/strength";
const String MQTT_TOPIC_ENVIRONMENT_TEMPERATURE_PATH = MQTT_TOPIC_BASE_PATH + "/environment/temperature";
const String MQTT_TOPIC_ENVIRONMENT_HUMIDITY_PATH = MQTT_TOPIC_BASE_PATH + "/environment/humidity";
const String MQTT_TOPIC_ERROR_REPORT_PATH = MQTT_TOPIC_BASE_PATH + "/error";

const char *DISPLAY_PREFERENCES_NAMESPACE = "display";
const char *DISPLAY_ETAG_KEY = "etag";

WiFiClient wiFiClient;
MQTTClient client(1024);

bool OTAEnabled = false;

void checkOTAStatus(String status)
{
  if (status == "1")
  {
    Serial.println("Device should switch to OTA ");
    OTAEnabled = true;
    setupOTA(HOSTNAME, OTA_PASSWORD);
  }
  else
  {
    Serial.println("Device is in normal status ");
    OTAEnabled = false;
  }
}

void messageReceived(String &topic, String &payload)
{
  Serial.println("incoming: " + topic + " - " + payload);
  if (topic == MQTT_TOPIC_OTA_CONFIG_PATH)
  {
    checkOTAStatus(payload);
  }
}

void connectToMQTT()
{
  client.begin(MQTT_SERVER, MQTT_SERVER_PORT, wiFiClient);
  client.onMessage(messageReceived);
  Serial.print("\nConnecting to mqtt...");
  while (!client.connect(HOSTNAME, MQTT_USERNAME, MQTT_PASSWORD))
  {
    Serial.print(".");
    delay(1000);
  }
  Serial.println("");
}

void subscribeToConfigChannel()
{
  client.subscribe(MQTT_TOPIC_OTA_CONFIG_PATH);
}

void publishHomeAssistantDiscovery()
{
  const String discoveryBase = "homeassistant/sensor/" + String(HOSTNAME);
  const String device = "\"device\":{\"identifiers\":[\"" + String(HOSTNAME) +
                        "\"],\"name\":\"Kathistiko\"}";

  // Remove the short-lived discovery identities used before the display
  // battery entity names were made explicit.
  client.publish(discoveryBase + "/battery_voltage/config", "", true, 1);
  client.publish(discoveryBase + "/battery_percentage/config", "", true, 1);

  const String voltageConfig =
      "{\"name\":\"Display battery voltage\",\"unique_id\":\"" + String(HOSTNAME) +
      "_display_battery_voltage\",\"default_entity_id\":\"sensor." + String(HOSTNAME) +
      "_display_battery_voltage\",\"state_topic\":\"" + MQTT_TOPIC_BATTERY_VOLTAGE_PATH +
      "\",\"device_class\":\"voltage\",\"state_class\":\"measurement\"," +
      "\"unit_of_measurement\":\"V\"," + device + "}";
  client.publish(discoveryBase + "/display_battery_voltage/config", voltageConfig, true, 1);

  const String percentageConfig =
      "{\"name\":\"Display battery\",\"unique_id\":\"" + String(HOSTNAME) +
      "_display_battery_percentage\",\"default_entity_id\":\"sensor." + String(HOSTNAME) +
      "_display_battery_percentage\",\"state_topic\":\"" + MQTT_TOPIC_BATTERY_PERCENTAGE_PATH +
      "\",\"device_class\":\"battery\",\"state_class\":\"measurement\"," +
      "\"unit_of_measurement\":\"%\"," + device + "}";
  client.publish(discoveryBase + "/display_battery_percentage/config", percentageConfig, true, 1);
}

void publishSensorsData(SensorData &data)
{
  const BatteryData battery = readBattery();

  client.publish(MQTT_TOPIC_WIFI_STRENGTH_PATH, String(getWifiStrength()));
  client.publish(MQTT_TOPIC_ENVIRONMENT_TEMPERATURE_PATH, String(data.temperature));
  client.publish(MQTT_TOPIC_ENVIRONMENT_HUMIDITY_PATH, String(data.humidity));
  client.publish(MQTT_TOPIC_BATTERY_VOLTAGE_PATH, String(battery.voltage, 3), true, 1);
  if (battery.percentageValid) {
    client.publish(MQTT_TOPIC_BATTERY_PERCENTAGE_PATH, String(battery.percentage), true, 1);
    Serial.printf("Battery: %.3f V (%u%% estimated)\n", battery.voltage,
                  static_cast<unsigned int>(battery.percentage));
  } else {
    client.publish(MQTT_TOPIC_BATTERY_PERCENTAGE_PATH, "None", true, 1);
    Serial.printf("Battery: %.3f V (outside valid percentage range)\n", battery.voltage);
  }
}
void reportDownloadError(String message)
{
  client.publish(MQTT_TOPIC_ERROR_REPORT_PATH, message);
}

// Function to go to deep sleep for a specified number of seconds
void goToSleep(int seconds)
{
  WiFi.disconnect(true);
  delay(1);
  Serial.println("ESP in sleep mode");
  esp_sleep_enable_timer_wakeup(seconds * 1000000);
  delay(100);
  esp_deep_sleep_start();
}

bool connectToWiFi()
{
  Serial.print("Connecting to ");
  Serial.println(WIFI_SSID);

  WiFi.mode(WIFI_STA);
  WiFi.setHostname("kathistiko");
  WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
  while (WiFi.waitForConnectResult() != WL_CONNECTED)
  {
    return false;
  }
  Serial.println("WiFi connected");
  const String ipAddress = getIp().toString();
  Serial.printf("IP address: %s\n", ipAddress.c_str());
  Serial.printf("Hostname: %s\n", WiFi.getHostname());
  Serial.println("Wifi Strength: " + String(getWifiStrength()) + " dB");

  return true;
}

IPAddress getIp()
{
  return WiFi.localIP();
}

int8_t getWifiStrength()
{
  int8_t rssi = WiFi.RSSI();

  return rssi;
}

void initDisplay()
{
  Serial.printf("Display pins: power=%d busy=%d rst=%d dc=%d cs=%d sck=%d miso=%d mosi=%d\n",
                POWER, BUSY, RST, DC, CS, SCK, MISO, MOSI);
  // Turn on power to the display
  pinMode(POWER, OUTPUT);
  digitalWrite(POWER, HIGH); // Turn the display power on (HIGH is the voltage level)
  delay(100);

  pinMode(BUSY, INPUT);
  pinMode(RST, OUTPUT);
  pinMode(DC, OUTPUT);
  pinMode(CS, OUTPUT);

  // SPI setup
  SPI.beginTransaction(SPISettings(10000000, MSBFIRST, SPI_MODE0));
  SPI.begin(SCK, MISO, MOSI, CS);
}

bool downloadAndRenderImage()
{
  Preferences preferences;
  String currentEtag;
  if (preferences.begin(DISPLAY_PREFERENCES_NAMESPACE, true)) {
    currentEtag = preferences.getString(DISPLAY_ETAG_KEY, "");
    preferences.end();
  } else {
    Serial.println("Failed to open display preferences for reading.");
  }

  size_t dataSize = 0;
  uint8_t *data = nullptr;
  String responseEtag;
  const ImageFetchResult result =
      fetchData(String(IMAGE_SERVER_URL), currentEtag, data, dataSize, responseEtag);

  if (result == ImageFetchResult::NotModified) {
    Serial.println("Display image is unchanged. Skipping download and refresh.");
    return true;
  }

  if (result == ImageFetchResult::Updated && data != nullptr && dataSize == EPD_ARRAY)
  {
    Serial.println("Data were successfully fetched. Going to render image on screen.");
    initDisplay();
    EPD_Init();
    EPD_WhiteScreen_ALL(data);
    EPD_DeepSleep();

    if (!responseEtag.isEmpty()) {
      if (preferences.begin(DISPLAY_PREFERENCES_NAMESPACE, false)) {
        if (preferences.putString(DISPLAY_ETAG_KEY, responseEtag) == 0) {
          Serial.println("Failed to persist display ETag.");
        }
        preferences.end();
      } else {
        Serial.println("Failed to open display preferences for writing.");
      }
    }

    delete[] data;
    return true;
  }

  delete[] data;
  Serial.println("Failed to fetch a valid display image.");
  return false;
}

ImageFetchResult fetchData(const String &url, const String &currentEtag,
                           uint8_t *&data, size_t &dataSize, String &responseEtag)
{
  HTTPClient http;
  const char *responseHeaders[] = {"ETag"};
  http.begin(url);
  http.collectHeaders(responseHeaders, 1);
  if (!currentEtag.isEmpty()) {
    http.addHeader("If-None-Match", currentEtag);
  }

  const int httpCode = http.GET();
  data = nullptr;
  dataSize = 0;
  responseEtag = "";

  if (httpCode <= 0) {
    Serial.printf("[HTTP] GET... failed, error: %s\n", http.errorToString(httpCode).c_str());
    http.end();
    return ImageFetchResult::Failed;
  }

  Serial.printf("[HTTP] GET... code: %d\n", httpCode);
  if (httpCode == HTTP_CODE_NOT_MODIFIED) {
    http.end();
    return ImageFetchResult::NotModified;
  }

  if (httpCode != HTTP_CODE_OK) {
    http.end();
    return ImageFetchResult::Failed;
  }

  const int contentLength = http.getSize();
  if (contentLength != EPD_ARRAY) {
    Serial.printf("Unexpected image size: %d bytes; expected %d.\n", contentLength, EPD_ARRAY);
    http.end();
    return ImageFetchResult::Failed;
  }

  data = new (std::nothrow) uint8_t[EPD_ARRAY];
  if (data == nullptr) {
    Serial.println("Failed to allocate display image buffer.");
    http.end();
    return ImageFetchResult::Failed;
  }

  WiFiClient *stream = http.getStreamPtr();
  unsigned long lastDataAt = millis();
  while (dataSize < EPD_ARRAY && (http.connected() || stream->available())) {
    const size_t available = stream->available();
    if (available > 0) {
      const size_t remaining = EPD_ARRAY - dataSize;
      const size_t requested = min(available, remaining);
      const size_t bytesRead = stream->readBytes(data + dataSize, requested);
      dataSize += bytesRead;
      lastDataAt = millis();
    } else {
      if (millis() - lastDataAt > 15000) {
        Serial.println("Timed out while downloading display image.");
        break;
      }
      delay(1);
    }
  }

  responseEtag = http.header("ETag");
  http.end();
  if (dataSize != EPD_ARRAY) {
    Serial.printf("Incomplete image: %u bytes; expected %d.\n",
                  static_cast<unsigned int>(dataSize), EPD_ARRAY);
    delete[] data;
    data = nullptr;
    dataSize = 0;
    return ImageFetchResult::Failed;
  }

  return ImageFetchResult::Updated;
}

// Function to print binary data in hexadecimal format
void printHexData(const uint8_t *data, size_t dataSize)
{
  Serial.println("Fetched binary data:");

  // Print the data in hexadecimal format for debugging
  for (size_t i = 0; i < dataSize; i++)
  {
    if (i % 16 == 0)
    {
      Serial.println(); // Print a newline every 16 bytes for readability
    }
    Serial.printf("%02X ", data[i]); // Print each byte in hexadecimal
  }
  Serial.println(); // Add a newline after printing all the data
}

void setup()
{
  Serial.begin(115200);
  Serial.println();
  Serial.println();
  Serial.println();
  Wire.begin(21, 22);
  for (uint8_t t = 4; t > 0; t--)
  {
    Serial.printf("[SETUP] WAIT %d...\n", t);
    Serial.flush();
    delay(1000);
  }
  Serial.println("Initialization.... ");

  // Connect to Wi-Fi
  if (!connectToWiFi())
  {
    Serial.println("Cannot connect to WiFi");
  }

  connectToMQTT();
  subscribeToConfigChannel();
  publishHomeAssistantDiscovery();
}

void doDisplayWork() {
  bool wasDownloaded = downloadAndRenderImage();
  if (wasDownloaded == false)
  {
    reportDownloadError("Failed to fetch new image");
  }
}

void doSensorsWork() {
  SensorData data;
  data = readSensors();
  publishSensorsData(data);
  // Do couple of loops to listen to subscribed channels
  client.loop();
  delay(3000);
  client.loop();
}

void loop()
{
  unsigned long currentMillis = millis();
  // Listening to the MQTT channels is configuring the device
  client.loop();
  delay(2);

  if (OTAEnabled == true)
  {
    OTAHandler();
    Serial.print("Listening on: http://");
    Serial.print(getIp());
    Serial.println(":3232");
    delay(2);
  }
  else
  {
    doSensorsWork();
    Serial.printf("currentMillis %lu\n", currentMillis);
    doDisplayWork();

    // If OTA got enabled, it has to be handled
    if (OTAEnabled == true)
    {
      OTAHandler();
      delay(2);
    }
    else
    {
      goToSleep(SLEEP_SECONDS);
    }
  }
}
/*
show_name: true
type: button
tap_action:
  action: toggle
show_icon: true
entity: binary_sensor.kathistiko_ota_config
name: Kathistiko OTA
*/
