# Kathistiko

A personal home dashboard and e-paper image backend built with Rust and Axum.
It displays calendar events, trash collection schedules, and sensor data while
serving the packed bitmap consumed by the Kathistiko ESP32 display.

## Features

- 📅 **Calendar Integration** - Display upcoming events from Apple Calendar (iCal format)
- 🗑️ **Trash Collection Reminders** - Show periodic trash and recycling collection schedules
- 🌡️ **Sensor Data** - Monitor temperature and other sensor readings
- 🎨 **Clean Web Interface** - Responsive dashboard with modern CSS styling
- ⚡ **Fast & Lightweight** - Built in Rust for optimal performance
- 🖥️ **E-paper Output** - Generates and serves the packed 800×480 bitmap used by the ESP32

## Unified Service

The dashboard and e-paper backend run in this single Axum service. A background
worker captures the local `/calendar` page through the snapshot container,
converts it to the display orientation, and publishes the 48,000-byte packed
bitmap at `/latest`.

Production routes:

- `/calendar` renders the browser dashboard.
- `/latest` serves the bitmap with an `ETag` and supports conditional
  `If-None-Match` requests from the ESP32.
- `/assets/converted_image.png` exposes the converted image for diagnostics.
- `/health/live` reports whether the HTTP service is running.
- `/health/ready` reports readiness after the first display image is available.

Rendering is configured in the `[display]` section. An optional
`display.local.toml` next to the main configuration file can override that
section without modifying a configuration containing credentials.

## Quick Start

### Prerequisites

- Rust 1.70+
- Configuration file (see `config.sample.toml`)

### Running Locally

1. Clone the repository
2. Install [just](https://github.com/casey/just) command runner (optional but recommended):
   ```bash
   # macOS
   brew install just

   # Or using cargo
   cargo install just
   ```
3. Copy and configure settings:
   ```bash
   # Using just
   just config-init
   # Edit config.local.toml with your settings

   # Or manually
   cp config.sample.toml config.local.toml
   ```
4. Run the application:
   ```bash
   # Using just
   just dev

   # Or using cargo directly
   cargo run
   ```
5. Open your browser to `http://localhost:8042` (or your configured port)

To see all available commands: `just --list`

### ESP32 firmware

The ESP32 firmware is maintained in the same repository as a PlatformIO project
using the Arduino framework. Its existing display, Wi-Fi, MQTT, sensor, deep
sleep, and OTA behavior is preserved.

See [`firmware/README.md`](firmware/README.md) for configuration, build, and
upload instructions. Device credentials belong only in
`firmware/include/conf.h`, which is ignored by Git.

### Configuration

The application uses TOML configuration files. Key settings include:

- `server_listen_at` - Server address and port
- Calendar data sources and endpoints
- Sensor data configuration
- Styling and display preferences

See `config.sample.toml` for all available options.

## Building for Production

### Local Build

```bash
# Using just (recommended)
just build-release

# Or using cargo directly
cargo build --release
```

### Cross-Compilation for Linux

```bash
# Using just (recommended)
just build-linux

# Or using cargo directly
cargo build --release --target x86_64-unknown-linux-gnu
```

For complete cross-compilation setup, see our [Cross-Compilation Guide](docs/cross-compilation.md).

## Project Structure

```
dashboard/
├── src/
│   ├── calendar/          # Calendar event handling
│   ├── sensor/            # Sensor data collection
│   ├── inbound/           # HTTP handlers and routing
│   └── settings.rs        # Configuration management
├── firmware/              # PlatformIO ESP32 firmware
├── templates/             # HTML templates (Askama)
├── public/               # Static assets (CSS, images)
├── docs/                 # Documentation
└── config.sample.toml    # Example configuration
```

## Technology Stack

- **Backend**: [Axum](https://github.com/tokio-rs/axum) web framework
- **Templates**: [Askama](https://github.com/djc/askama) template engine
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest) with rustls
- **Calendar**: iCalendar parsing with [icalendar](https://github.com/hoodie/icalendar-rs)
- **Config**: [config-rs](https://github.com/mehcode/config-rs) for TOML configuration
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Firmware**: PlatformIO with the Arduino framework for ESP32

## Documentation

- [Architecture Overview](docs/architecture.md) - System design and Prometheus integration
- [Cross-Compilation Guide](docs/cross-compilation.md) - Build Linux binaries from macOS
- [Deployment Guide](docs/deployment.md) - Production server deployment and management
- [Firmware Guide](firmware/README.md) - ESP32 configuration, build, and upload
- [Configuration Reference](config.sample.toml) - All available configuration options

## License

This project is licensed under the terms found in the [LICENSE](LICENSE) file.

## Contributing

This is a personal dashboard project, but feel free to fork it and adapt it to your needs!
