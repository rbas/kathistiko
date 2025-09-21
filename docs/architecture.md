# Architecture Overview

This document describes the architecture of the Dashboard application, including its components, data flow, and integration with external systems like Prometheus and Home Assistant.

## System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│  Web Browser    │────│  Dashboard App  │────│  External APIs  │
│                 │    │   (Axum/Rust)   │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              │
                    ┌─────────┴─────────┐
                    │                   │
          ┌─────────▼─────────┐ ┌───────▼───────┐
          │                   │ │               │
          │  Calendar Data    │ │ Sensor Data   │
          │  (iCal feeds)     │ │ (Prometheus)  │
          │                   │ │               │
          └───────────────────┘ └───────────────┘
```

## Components Overview

### 1. Web Server Layer (`src/inbound/`)
- **HTTP Server**: Built with [Axum](https://github.com/tokio-rs/axum) framework
- **Request Handlers**: Process incoming HTTP requests
- **Template Rendering**: Uses [Askama](https://github.com/djc/askama) for HTML generation
- **Static File Serving**: Serves CSS and other assets

### 2. Calendar System (`src/calendar/`)
- **iCal Parser**: Downloads and parses calendar feeds
- **Event Processing**: Filters and formats calendar events
- **Trash Schedule**: Generates periodic trash collection reminders
- **Caching**: Local file-based caching with TTL

### 3. Sensor System (`src/sensor/`)
- **Prometheus Integration**: Fetches sensor data from Prometheus/Home Assistant
- **Data Models**: Type-safe representation of temperature and humidity sensors
- **Repository Pattern**: Abstracts data access from business logic
- **Error Handling**: Robust error handling with graceful degradation

### 4. Configuration (`src/settings.rs`)
- **TOML Configuration**: File-based configuration management
- **Environment Isolation**: Separate configs for development/production
- **Validation**: Configuration validation at startup

## Data Flow

### Request Lifecycle

1. **HTTP Request** arrives at the Axum web server
2. **Route Handler** (`calendar_handler`) is invoked
3. **Parallel Data Fetching**:
   - Calendar events from iCal URLs
   - Sensor data from Prometheus
   - Trash schedule generation
4. **Data Processing**:
   - Parse and filter calendar events
   - Convert sensor data to typed models
   - Handle errors gracefully (partial data ok)
5. **Template Rendering**:
   - Combine all data into template context
   - Render HTML using Askama
6. **HTTP Response** sent to browser

```
HTTP Request → Handler → ┌─ Calendar API
                        ├─ Prometheus API  → Template → HTML Response
                        └─ Trash Generator
```

## Prometheus Integration

### Overview

The Dashboard integrates with Prometheus to collect sensor data from Home Assistant. This provides real-time temperature and humidity readings from multiple locations.

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│ Home Assistant  │────│   Prometheus    │────│   Dashboard     │
│                 │    │                 │    │                 │
│ - Sensors       │    │ - Metrics       │    │ - Query API     │
│ - Devices       │    │ - Time Series   │    │ - Data Models   │
│ - Integrations  │    │ - Storage       │    │ - Web Display   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Data Flow

1. **Home Assistant** collects sensor data from physical devices
2. **Prometheus** scrapes metrics from Home Assistant at regular intervals
3. **Dashboard** queries Prometheus API for latest sensor readings
4. **Data Processing** converts raw metrics to typed sensor models
5. **Web Display** shows current temperature/humidity on dashboard

### Prometheus Query

The Dashboard uses a specific PromQL query to fetch sensor data:

```
{__name__=~"hass_sensor_unit_u0x25u0x20rh|hass_sensor_unit_celsius",entity=~"sensor.kathistiko_humidity|sensor.kairos_humidity|sensor.kathistiko_temperature|sensor.kairos_temperature"}
```

This query:
- Filters metrics by unit type (celsius for temperature, %rh for humidity)
- Targets specific sensor entities from two locations:
  - **Kathistiko**: Indoor/living room sensors
  - **Kairos**: Outdoor sensors

### Sensor Models

#### Raw Data Structure
```rust
struct ApiResponse {
    data: Data {
        result: Vec<ResultItem {
            metric: Metric {
                entity: String,  // e.g., "sensor.kathistiko_temperature"
            },
            value: (f64, String), // (timestamp, value)
        }>
    }
}
```

#### Processed Models
```rust
enum SensorName {
    KairosTemperature,     // Outdoor temperature
    KairosHumidity,        // Outdoor humidity
    KathistikoTemperature, // Indoor temperature
    KathistikoHumidity,    // Indoor humidity
}

struct SensorData {
    name: SensorName,
    value: f32,
    captured_at: DateTime<Utc>,
}

struct LivingRoom {
    temperature_sensor: SensorData, // Kathistiko sensors
    humidity_sensor: SensorData,
}

struct Outdoor {
    temperature_sensor: SensorData, // Kairos sensors
    humidity_sensor: SensorData,
}
```

### Error Handling

The system implements graceful degradation for sensor data:

1. **Network Failures**: If Prometheus is unreachable, dashboard continues without sensor data
2. **Authentication Errors**: Invalid credentials are logged, dashboard shows without sensors
3. **Missing Sensors**: If specific sensors are unavailable, only available ones are shown
4. **Data Parsing**: Invalid sensor values are logged and skipped

### Configuration

Prometheus integration requires these configuration parameters:

```toml
prometheus_url = "http://prometheus.local:9090"
prometheus_username = "dashboard"
prometheus_password = "secure_password"
```

### Security

- **HTTP Basic Authentication**: Used for Prometheus API access
- **Credential Management**: Stored in configuration files (not in code)
- **Network Security**: Typically deployed on internal networks only
