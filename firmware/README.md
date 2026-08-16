# ESP32 firmware

PlatformIO firmware for the LaskaKit ESPink ESP32 e-paper board (v2.x) and the
Good Display GDEY075T7 800x480 display.

## Configure

Copy the example configuration and replace every placeholder:

```sh
cp include/conf.example.h include/conf.h
```

`include/conf.h` contains Wi-Fi, OTA, MQTT, and image-service settings. It is
ignored by Git. Do not put private calendar URLs in the firmware; the device
only needs the bitmap endpoint.

## Build and upload

```sh
pio run
pio run --target upload
pio device monitor
```

The initial migration continues to use the public `/latest` endpoint. Once the
new firmware is verified on the device, set `IMAGE_SERVER_URL` in `conf.h` to
the service's internal-network address.

## Battery telemetry

ESPink v2.x battery voltage is sampled on GPIO 34 through the board's 1.769388
voltage divider. The firmware averages 16 calibrated ADC readings and publishes
retained MQTT values to:

- `/home/<hostname>/battery/voltage`
- `/home/<hostname>/battery/percentage`

The percentage is an estimate derived from a typical single-cell Li-Po discharge
curve; voltage is retained separately for diagnostics and calibration. Firmware
also publishes Home Assistant MQTT discovery records for both sensors. The
dashboard displays the percentage in 5% steps as small `BAT 80%` text in the bottom-right
corner when `sensor.<hostname>_display_battery_percentage` is available through
its Prometheus data source.

## Conditional display refresh

Every 30-minute wake publishes sensor telemetry and conditionally requests
`/latest`. The service identifies the exact packed bitmap with an HTTP `ETag`.
The firmware stores the ETag in ESP32 NVS and sends it in `If-None-Match` on the
next wake. A `304 Not Modified` response skips the 48,000-byte download and does
not power the e-paper panel. A changed image is downloaded, validated, rendered,
and only then recorded as the displayed ETag.
