# Cross-Compilation Guide

This guide explains how to cross-compile the Dashboard application for Linux deployment from macOS.

## Setup

This project is configured for cross-compilation from macOS to Linux (`x86_64-unknown-linux-gnu` target). Below are the requirements and setup instructions.

### Prerequisites

#### 1. Install Cross-Compilation Toolchain

You need a cross-compilation toolchain for Linux. Install it using Homebrew:

```bash
brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu
```

This installs the necessary GCC cross-compiler and associated tools.

#### 2. Add Rust Target

Add the Linux target to your Rust installation:

```bash
rustup target add x86_64-unknown-linux-gnu
```

### Configuration Files

#### `.cargo/config.toml`

The project includes a Cargo configuration file that specifies the correct linker for cross-compilation:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"
```

This file is already included in the repository and should not be modified.

#### `Cargo.toml` Dependencies

The project uses `rustls` instead of `native-tls` (OpenSSL) to avoid cross-compilation complexities:

```toml
reqwest = { version = "0.12.15", features = ["json", "rustls-tls"], default-features = false }
```

### What to Avoid

❌ **Don't use OpenSSL-based dependencies** for cross-compilation:

- `native-tls` feature in `reqwest`
- `openssl` crate directly
- Any crate that depends on system OpenSSL libraries

❌ **Don't use these alternative toolchains** (they may conflict):

- `sergiobenitez/osxct` Homebrew tap
- Manual OpenSSL compilation for cross-targets

### Building for Linux

#### Development Build

```bash
cargo build --target x86_64-unknown-linux-gnu
```

#### Release Build

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

The resulting binary will be located at:

```
target/x86_64-unknown-linux-gnu/release/dashboard
```

This self-contained binary can be deployed directly to any Linux system with glibc. No additional SSL libraries need to be installed on the target system since we use `rustls`.

For complete deployment instructions, see the [Deployment Guide](deployment.md).

### Troubleshooting

#### Common Issues

1. **"Could not find directory of OpenSSL installation"**
   - This means you're using a dependency with `native-tls`
   - Switch to `rustls-tls` features instead
   - See the `reqwest` configuration in `Cargo.toml` as an example

2. **Linker errors with `rust-lld`**
   - Ensure `.cargo/config.toml` specifies the correct linker
   - Verify the cross-compilation toolchain is installed

3. **"unknown argument" linker errors**
   - Usually indicates missing or incorrect linker configuration
   - Make sure you're using `x86_64-unknown-linux-gnu-gcc` as the linker

#### Verification Commands

Check if the toolchain is properly installed:

```bash
which x86_64-unknown-linux-gnu-gcc
# Should output: /opt/homebrew/bin/x86_64-unknown-linux-gnu-gcc
```

Check if the Rust target is available:

```bash
rustup target list | grep x86_64-unknown-linux-gnu
# Should show: x86_64-unknown-linux-gnu (installed)
```

### Development

For local development on macOS, you can use the standard commands:

```bash
cargo run          # Run locally
cargo test         # Run tests
cargo check        # Quick compile check
```

### Dependencies

The project uses `rustls` for TLS operations, which provides:

- ✅ Pure Rust implementation (no C dependencies)
- ✅ Easier cross-compilation
- ✅ Smaller binary size
- ✅ No system SSL library dependencies
- ✅ Modern TLS implementation

This choice eliminates the need to cross-compile OpenSSL and ensures the binary works on any Linux system without additional SSL library installations.
