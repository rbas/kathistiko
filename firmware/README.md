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
