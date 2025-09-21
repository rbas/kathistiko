# Dashboard Project Commands
# Use 'just --list' to see all available commands

# Default recipe when 'just' is run without arguments
default:
    @just --list

# === Development Commands ===

# Run the application locally for development
dev:
    cargo run

# Run with file watching for development
dev-watch:
    cargo watch -x run

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Clean build artifacts
clean:
    cargo clean

# === Build Commands ===

# Build for local development
build:
    cargo build

# Build optimized release version
build-release:
    cargo build --release

# Build for Linux deployment (cross-compilation)
build-linux: _ensure-linux-target
    #!/usr/bin/env bash
    echo "🔨 Building for Linux (x86_64-unknown-linux-gnu)..."
    cargo build --release --target x86_64-unknown-linux-gnu
    echo "✅ Linux binary built: target/x86_64-unknown-linux-gnu/release/dashboard"

# Check if Linux target is installed, install if needed
_ensure-linux-target:
    #!/usr/bin/env bash
    if ! rustup target list --installed | grep -q x86_64-unknown-linux-gnu; then
        echo "📦 Installing Linux target..."
        rustup target add x86_64-unknown-linux-gnu
    fi

# === Deployment Commands ===

# Deploy to production server (builds + uploads + restarts)
deploy: build-linux _deploy-files _restart-service
    @echo "🚀 Deployment completed successfully!"

# Deploy without building (use existing binary)
deploy-only: _deploy-files _restart-service
    @echo "🚀 Deployment completed successfully!"

# Update code and deploy
deploy-full: _git-pull deploy
    @echo "🚀 Full deployment with git pull completed!"

# Upload files to server
_deploy-files:
    #!/usr/bin/env bash
    echo "📤 Uploading files to nabu server..."
    scp target/x86_64-unknown-linux-gnu/release/dashboard rbas@nabu:/srv/kathistiko/dashboard/
    scp config.sample.toml rbas@nabu:/srv/kathistiko/dashboard/
    scp public/css/main.css rbas@nabu:/srv/kathistiko/dashboard/public/css/
    echo "✅ Files uploaded successfully"

# Restart service on remote server
_restart-service:
    #!/usr/bin/env bash
    echo "🔄 Restarting service on remote server..."
    ssh -t rbas@nabu 'sudo systemctl restart kathistikodashboard.service'
    echo "✅ Service restarted successfully"

# Pull latest changes from git
_git-pull:
    #!/usr/bin/env bash
    echo "📡 Pulling latest changes..."
    git pull
    echo "✅ Git pull completed"

# === Utility Commands ===

# Show binary size information
size:
    #!/usr/bin/env bash
    echo "📊 Binary sizes:"
    if [ -f "target/release/dashboard" ]; then
        echo "Local release: $(ls -lh target/release/dashboard | awk '{print $5}')"
    fi
    if [ -f "target/x86_64-unknown-linux-gnu/release/dashboard" ]; then
        echo "Linux release: $(ls -lh target/x86_64-unknown-linux-gnu/release/dashboard | awk '{print $5}')"
    fi

# Check server status
status:
    #!/usr/bin/env bash
    echo "🔍 Checking server status..."
    ssh -t rbas@nabu 'sudo systemctl status kathistikodashboard.service'

# View server logs
logs lines="50":
    #!/usr/bin/env bash
    echo "📋 Viewing last {{lines}} lines of server logs..."
    ssh -t rbas@nabu 'sudo journalctl -u kathistikodashboard.service -n {{lines}} --no-pager'

# Follow server logs in real-time
logs-follow:
    #!/usr/bin/env bash
    echo "📋 Following server logs (Ctrl+C to stop)..."
    ssh -t rbas@nabu 'sudo journalctl -u kathistikodashboard.service -f'

# Alternative: View logs without sudo (if user is in systemd-journal group)
logs-nosudo lines="50":
    #!/usr/bin/env bash
    echo "📋 Viewing last {{lines}} lines of server logs (no sudo)..."
    ssh rbas@nabu 'journalctl --user-unit kathistikodashboard.service -n {{lines}} --no-pager 2>/dev/null || echo "❌ No user logs found. Use regular logs command with sudo."'

# Alternative: Check if server is responding via HTTP
ping-server:
    #!/usr/bin/env bash
    echo "🏓 Checking if dashboard is responding..."
    if curl -f -s -o /dev/null http://nabu:8042; then
        echo "✅ Dashboard is responding"
    else
        echo "❌ Dashboard is not responding"
        exit 1
    fi

# === Configuration Commands ===

# Validate configuration syntax
config-check file="config.local.toml":
    #!/usr/bin/env bash
    if [ -f "{{file}}" ]; then
        echo "✅ Checking configuration file: {{file}}"
        # Basic TOML syntax check using a simple test
        if cargo run -- --config {{file}} --help > /dev/null 2>&1; then
            echo "✅ Configuration appears valid"
        else
            echo "❌ Configuration may have issues"
            exit 1
        fi
    else
        echo "❌ Configuration file not found: {{file}}"
        exit 1
    fi

# Create configuration from sample
config-init:
    #!/usr/bin/env bash
    if [ ! -f "config.local.toml" ]; then
        cp config.sample.toml config.local.toml
        echo "✅ Created config.local.toml from sample"
        echo "📝 Please edit config.local.toml with your settings"
    else
        echo "⚠️  config.local.toml already exists"
    fi

# === Documentation Commands ===

# Serve documentation locally (if you add mdBook later)
docs:
    @echo "📚 Documentation available in docs/ folder"
    @echo "📖 Main files:"
    @echo "   - README.md"
    @echo "   - docs/architecture.md"
    @echo "   - docs/cross-compilation.md" 
    @echo "   - docs/deployment.md"

# Open documentation in browser
docs-open:
    #!/usr/bin/env bash
    if command -v open > /dev/null; then
        open README.md
    elif command -v xdg-open > /dev/null; then
        xdg-open README.md
    else
        echo "Please open README.md in your preferred editor/browser"
    fi

# === Release Commands ===

# Create a release using automatic version bump
release:
    #!/usr/bin/env bash
    set -euxo pipefail
    echo "⏳ Pulling latest changes..."
    git pull origin main
    echo "🚀 Creating release (auto-detected bump based on commits)..."
    git cliff --bump -o CHANGELOG.md
    RELEASE_VERSION=$(git cliff --bumped-version)
    # Remove 'v' prefix for Cargo.toml (git cliff outputs "v1.2.3", we need "1.2.3")
    CARGO_VERSION=${RELEASE_VERSION#v}
    echo "✅ Release version: ${RELEASE_VERSION}"
    echo "📝 Updating Cargo.toml version to ${CARGO_VERSION}..."
    sed -i.bak "s/^version = \".*\"/version = \"${CARGO_VERSION}\"/" Cargo.toml
    rm Cargo.toml.bak
    echo "🔄 Updating Cargo.lock with new version..."
    cargo update --package dashboard --precise ${CARGO_VERSION}
    echo "💾 Staging changelog, Cargo.toml, and Cargo.lock..."
    git add CHANGELOG.md Cargo.toml Cargo.lock
    echo "📝 Committing changelog and version bump..."
    git commit -m "chore(release): update changelog and version for $RELEASE_VERSION"
    echo "📤 Pushing commit to main branch..."
    git push origin main
    echo "🏷 Creating and pushing tag ${RELEASE_VERSION}..."
    git tag $RELEASE_VERSION
    git push origin $RELEASE_VERSION
    echo "🎉 Release complete!"

# Preview what the next release would be
release-preview:
    #!/usr/bin/env bash
    echo "🔮 Previewing next release..."
    NEXT_VERSION=$(git cliff --bumped-version)
    NEXT_CARGO_VERSION=${NEXT_VERSION#v}
    CURRENT_CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo "📊 Next version would be: ${NEXT_VERSION}"
    echo "📦 Cargo.toml version: ${CURRENT_CARGO_VERSION} → ${NEXT_CARGO_VERSION}"
    echo ""
    echo "📋 Changelog preview:"
    git cliff --unreleased --strip header

# Generate changelog without creating a release
changelog:
    #!/usr/bin/env bash
    echo "📋 Generating changelog..."
    git cliff -o CHANGELOG.md
    echo "✅ Changelog updated in CHANGELOG.md"

# Check version consistency between git tags and Cargo.toml
version-check:
    #!/usr/bin/env bash
    echo "🔍 Checking version consistency..."
    CURRENT_CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    LATEST_GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "no-tags")
    LATEST_TAG_VERSION=${LATEST_GIT_TAG#v}
    
    echo "📦 Cargo.toml version: ${CURRENT_CARGO_VERSION}"
    echo "🏷️  Latest git tag: ${LATEST_GIT_TAG}"
    
    if [ "$CURRENT_CARGO_VERSION" = "$LATEST_TAG_VERSION" ]; then
        echo "✅ Versions are in sync!"
    else
        echo "⚠️  Versions are out of sync!"
        echo "   Consider running 'just release' to create a new release"
    fi

# Manually update Cargo.toml version (useful for development)
version-set version:
    #!/usr/bin/env bash
    echo "📝 Setting Cargo.toml version to {{version}}..."
    sed -i.bak "s/^version = \".*\"/version = \"{{version}}\"/" Cargo.toml
    rm Cargo.toml.bak
    echo "🔄 Updating Cargo.lock with new version..."
    cargo update --package dashboard --precise {{version}}
    echo "✅ Cargo.toml and Cargo.lock updated to {{version}}"
    echo "💡 Don't forget to commit these changes if desired"

# === Environment Commands ===

# Show environment information
env:
    #!/usr/bin/env bash
    echo "🔧 Development Environment:"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo "Just version: $(just --version)"
    echo ""
    echo "📦 Installed targets:"
    rustup target list --installed
    echo ""
    echo "🛠️  Development tools:"
    if command -v x86_64-unknown-linux-gnu-gcc > /dev/null; then
        echo "✅ x86_64-unknown-linux-gnu-gcc: $(which x86_64-unknown-linux-gnu-gcc)"
    else
        echo "❌ x86_64-unknown-linux-gnu-gcc not found"
    fi
    if command -v git-cliff > /dev/null; then
        echo "✅ git-cliff: $(git-cliff --version)"
    else
        echo "❌ git-cliff not found (install with: cargo install git-cliff)"
    fi

# Install development dependencies
install-deps:
    #!/usr/bin/env bash
    echo "📦 Installing development dependencies..."
    
    # Check and install cargo-watch
    if ! command -v cargo-watch > /dev/null; then
        echo "Installing cargo-watch..."
        cargo install cargo-watch
    fi
    
    # Check and install git-cliff
    if ! command -v git-cliff > /dev/null; then
        echo "Installing git-cliff..."
        cargo install git-cliff
    fi
    
    # Check cross-compilation toolchain
    if ! command -v x86_64-unknown-linux-gnu-gcc > /dev/null; then
        echo "❌ Cross-compilation toolchain not found"
        echo "Please install it with:"
        echo "  brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu"
    else
        echo "✅ Cross-compilation toolchain already installed"
    fi
    
    echo "✅ Development environment setup complete"
