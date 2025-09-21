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
