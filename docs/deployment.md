# Deployment Guide

This guide covers deploying the Dashboard application to production servers.

## Prerequisites

- A cross-compiled Linux binary (see [Cross-Compilation Guide](cross-compilation.md))
- Target Linux server with glibc support
- SSH access to the target server
- [just](https://github.com/casey/just) command runner (recommended for easier deployment)

## Binary Deployment

The Dashboard application is compiled as a self-contained binary with no external dependencies thanks to `rustls`. This makes deployment straightforward.

### Transfer Binary to Server

First, build the Linux binary:
```bash
# Using just (recommended)
just build-linux

# Or using cargo directly
cargo build --release --target x86_64-unknown-linux-gnu
```

Then transfer to server:
```bash
scp target/x86_64-unknown-linux-gnu/release/dashboard user@server:/path/to/deployment/
```

### Set Executable Permissions

```bash
ssh user@server "chmod +x /path/to/deployment/dashboard"
```

### Automated Deployment (Recommended)

If you have [just](https://github.com/casey/just) configured, you can use the automated deployment commands:

```bash
# Full deployment: build + upload + restart service
just deploy

# Deploy without rebuilding (uses existing binary)
just deploy-only

# Full deployment with git pull
just deploy-full
```

**Note**: The `just deploy` commands are configured for the specific server setup (`rbas@nabu`). You'll need to modify the `justfile` to match your server configuration.

## Configuration

### Transfer Configuration File

The application requires a configuration file. Transfer your production config:

```bash
scp config.production.toml user@server:/path/to/deployment/config.local.toml
```

### Configuration Location

The application looks for configuration in this order:
1. Path specified with `-c` or `--config` flag
2. `config.local.toml` in the current directory
3. Default fallback values

### Sample Production Configuration

```toml
# Server configuration
server_listen_at = "0.0.0.0:8042"

# Calendar sources
[calendar]
sources = [
    "https://your-calendar-url.com/calendar.ics"
]

# Sensor endpoints
[sensors]
prometheus_endpoint = "http://your-prometheus:9090"

# Add other production-specific settings...
```

## Running the Application

### Manual Execution

```bash
cd /path/to/deployment
./dashboard
```

### With Custom Config

```bash
./dashboard --config /path/to/your/config.toml
```

### Background Execution

```bash
nohup ./dashboard > dashboard.log 2>&1 &
```

## Service Management

### Kathistiko production units

The repository contains the production dashboard and Prometheus units in
`deploy/systemd`. Install them with:

```bash
just install-systemd
```

This command requires an interactive sudo password. It preserves the existing
units as `*.pre-kathistiko-fix`, enables a persistent user runtime for rootless
Podman, and restarts both services. The persistent runtime is required because
the dashboard starts the snapshot container between SSH sessions.

### Systemd Service (Recommended)

Create a systemd service file for proper process management:

```ini
# /etc/systemd/system/dashboard.service
[Unit]
Description=Personal Dashboard
After=network.target

[Service]
Type=simple
User=dashboard
Group=dashboard
WorkingDirectory=/opt/dashboard
ExecStart=/opt/dashboard/dashboard
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable dashboard
sudo systemctl start dashboard
```

Check service status:

```bash
sudo systemctl status dashboard
sudo journalctl -u dashboard -f  # Follow logs
```

### Process Management Commands

```bash
# Start service
sudo systemctl start dashboard

# Stop service
sudo systemctl stop dashboard

# Restart service
sudo systemctl restart dashboard

# View logs
sudo journalctl -u dashboard -n 100
```

## Reverse Proxy Setup

### Nginx Configuration

For production deployments, use a reverse proxy:

```nginx
# /etc/nginx/sites-available/dashboard
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:8042;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Serve static files directly
    location /css/ {
        proxy_pass http://127.0.0.1:8042;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/dashboard /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### SSL/HTTPS with Let's Encrypt

```bash
sudo certbot --nginx -d your-domain.com
```

## Security Considerations

### User Isolation

Create a dedicated user for the service:

```bash
sudo useradd --system --home /opt/dashboard --shell /bin/false dashboard
sudo mkdir -p /opt/dashboard
sudo chown dashboard:dashboard /opt/dashboard
```

### File Permissions

```bash
# Application binary
sudo chmod 755 /opt/dashboard/dashboard
sudo chown dashboard:dashboard /opt/dashboard/dashboard

# Configuration file
sudo chmod 600 /opt/dashboard/config.local.toml
sudo chown dashboard:dashboard /opt/dashboard/config.local.toml
```

### Firewall Configuration

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp   # SSH
sudo ufw allow 80/tcp   # HTTP
sudo ufw allow 443/tcp  # HTTPS
sudo ufw enable

# Block direct access to application port
sudo ufw deny 8042/tcp
```

## Monitoring and Maintenance

### Log Management

Configure log rotation:

```bash
# /etc/logrotate.d/dashboard
/var/log/dashboard/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 dashboard dashboard
    postrotate
        systemctl reload dashboard
    endscript
}
```

### Health Checks

Simple health check script:

```bash
#!/bin/bash
# /opt/dashboard/health-check.sh

ENDPOINT="http://localhost:8042"
if curl -f -s "$ENDPOINT" > /dev/null; then
    echo "Dashboard is healthy"
    exit 0
else
    echo "Dashboard is not responding"
    exit 1
fi
```

### Backup Configuration

```bash
#!/bin/bash
# /opt/dashboard/backup-config.sh

BACKUP_DIR="/backup/dashboard"
CONFIG_FILE="/opt/dashboard/config.local.toml"

mkdir -p "$BACKUP_DIR"
cp "$CONFIG_FILE" "$BACKUP_DIR/config-$(date +%Y%m%d-%H%M%S).toml"

# Keep only last 30 backups
find "$BACKUP_DIR" -name "config-*.toml" -mtime +30 -delete
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   sudo chown dashboard:dashboard /opt/dashboard/dashboard
   sudo chmod +x /opt/dashboard/dashboard
   ```

2. **Port Already in Use**
   ```bash
   sudo netstat -tlnp | grep :8042
   sudo systemctl stop conflicting-service
   ```

3. **Configuration Not Found**
   ```bash
   # Check config file location and permissions
   ls -la /opt/dashboard/config.local.toml
   ```

4. **Service Won't Start**
   ```bash
   sudo journalctl -u dashboard -n 50
   sudo systemctl status dashboard
   ```

### Log Analysis

```bash
# Real-time logs
sudo journalctl -u dashboard -f

# Error logs only
sudo journalctl -u dashboard -p err

# Logs from last hour
sudo journalctl -u dashboard --since "1 hour ago"
```

## Updates

### Deploying New Versions

1. Build new binary locally (see [Cross-Compilation Guide](cross-compilation.md))
2. Stop the service:
   ```bash
   sudo systemctl stop dashboard
   ```
3. Backup current binary:
   ```bash
   sudo cp /opt/dashboard/dashboard /opt/dashboard/dashboard.backup
   ```
4. Upload new binary:
   ```bash
   scp target/x86_64-unknown-linux-gnu/release/dashboard user@server:/tmp/
   sudo mv /tmp/dashboard /opt/dashboard/dashboard
   sudo chown dashboard:dashboard /opt/dashboard/dashboard
   sudo chmod +x /opt/dashboard/dashboard
   ```
5. Start the service:
   ```bash
   sudo systemctl start dashboard
   ```
6. Verify deployment:
   ```bash
   sudo systemctl status dashboard
   curl http://localhost:8042
   ```

### Rollback Process

If something goes wrong:

```bash
sudo systemctl stop dashboard
sudo mv /opt/dashboard/dashboard.backup /opt/dashboard/dashboard
sudo systemctl start dashboard
```
