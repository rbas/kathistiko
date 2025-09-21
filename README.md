# Dashboard

A personal home dashboard application built with Rust and Axum that displays calendar events, trash collection schedules, and sensor data in a clean web interface.

## Features

- 📅 **Calendar Integration** - Display upcoming events from Apple Calendar (iCal format)
- 🗑️ **Trash Collection Reminders** - Show periodic trash and recycling collection schedules
- 🌡️ **Sensor Data** - Monitor temperature and other sensor readings
- 🎨 **Clean Web Interface** - Responsive dashboard with modern CSS styling
- ⚡ **Fast & Lightweight** - Built in Rust for optimal performance

## Quick Start

### Prerequisites

- Rust 1.70+
- Configuration file (see `config.sample.toml`)

### Running Locally

1. Clone the repository
2. Copy and configure settings:
   ```bash
   cp config.sample.toml config.local.toml
   # Edit config.local.toml with your settings
   ```
3. Run the application:
   ```bash
   cargo run
   ```
4. Open your browser to `http://localhost:8042` (or your configured port)

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
cargo build --release
```

### Cross-Compilation for Linux

For building Linux binaries from macOS, see our [Cross-Compilation Guide](docs/cross-compilation.md).

## Project Structure

```
dashboard/
├── src/
│   ├── calendar/          # Calendar event handling
│   ├── sensor/            # Sensor data collection
│   ├── inbound/           # HTTP handlers and routing
│   └── settings.rs        # Configuration management
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

## Documentation

- [Architecture Overview](docs/architecture.md) - System design and Prometheus integration
- [Cross-Compilation Guide](docs/cross-compilation.md) - Build Linux binaries from macOS
- [Deployment Guide](docs/deployment.md) - Production server deployment and management
- [Configuration Reference](config.sample.toml) - All available configuration options

## License

This project is licensed under the terms found in the [LICENSE](LICENSE) file.

## Contributing

This is a personal dashboard project, but feel free to fork it and adapt it to your needs!
