# Default recipe
default: build

# Build the project in release mode
build:
    #!/usr/bin/env bash
    set -e
    echo "🔨 Building pcli and pdaemon..."
    cargo build --release
    echo "✅ Build completed successfully!"

# Install binaries to ~/.local/bin/
install: build
    #!/usr/bin/env bash
    set -e
    
    echo "📦 Installing binaries..."
    
    # Create directory if it doesn't exist
    mkdir -p ~/.local/bin
    
    # Install pdaemon
    echo "⚡ Installing pdaemon..."
    install -Dm755 ./target/release/pdaemon ~/.local/bin/pdaemon
    
    # Install pcli  
    echo "⚡ Installing pcli..."
    install -Dm755 ./target/release/pcli ~/.local/bin/pcli
    
    echo "✅ Installation completed!"
    echo "📍 Binaries installed to:"
    echo "   - ~/.local/bin/pdaemon"
    echo "   - ~/.local/bin/pcli"

# Restart the daemon
restart-daemon:
    #!/usr/bin/env bash
    set -e
    
    echo "🔄 Restarting pdaemon..."
    
    # Kill existing daemon
    if pgrep -x "pdaemon" > /dev/null; then
        echo "⏹️  Stopping existing pdaemon..."
        pkill pdaemon
        sleep 0.5
    else
        echo "ℹ️  No existing pdaemon process found"
    fi
    
    # Start new daemon
    echo "▶️  Starting new pdaemon..."
    pdaemon & disown
    
    sleep 0.5
    
    # Verify it's running
    if pgrep -x "pdaemon" > /dev/null; then
        echo "✅ pdaemon started successfully!"
    else
        echo "❌ Failed to start pdaemon"
        exit 1
    fi

# Full install and restart
install-and-restart: install restart-daemon

# Clean build artifacts
clean:
    echo "🧹 Cleaning build artifacts..."
    cargo clean
    echo "✅ Clean completed!"

# Development build (debug mode)
dev:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Building in debug mode..."
    cargo build
    echo "✅ Debug build completed!"

# Run tests
test:
    echo "🧪 Running tests..."
    cargo test

# Check code formatting
check:
    #!/usr/bin/env bash
    set -e
    echo "🔍 Checking code..."
    cargo check
    cargo clippy -- -D warnings
    echo "✅ Code checks passed!"

# Show help
help:
    echo "Available commands:"
    echo "  build           - Build the project in release mode"
    echo "  install         - Build and install binaries to ~/.local/bin/"
    echo "  restart-daemon  - Restart the pdaemon service"
    echo "  install-and-restart - Install binaries and restart daemon"
    echo "  clean           - Clean build artifacts"
    echo "  dev             - Build in debug mode"
    echo "  test            - Run tests"
    echo "  check           - Run code checks (cargo check + clippy)"
    echo "  help            - Show this help message"
