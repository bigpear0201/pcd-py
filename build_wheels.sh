#!/bin/bash
set -e

echo "Building pcd-py wheels..."

# Check for required tools
if ! command -v maturin &> /dev/null; then
    echo "Error: maturin is not installed. Please install it with 'pip install maturin'."
    exit 1
fi

# Ensure output directory exists
mkdir -p dist

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)

echo "Platform: $PLATFORM $ARCH"

# Check if using path dependency (development mode)
if grep -q 'path = "../pcd-rs"' Cargo.toml; then
    echo "⚠️  WARNING: Cargo.toml uses path dependency to pcd-rs"
    echo "   This will NOT work for building distributable wheels!"
    echo "   Please update to use a published version or git dependency:"
    echo "   rs-pcd = { version = \"0.2.0\", features = [\"memmap2\", \"rayon\"] }"
    echo ""
    read -p "Continue anyway (for local testing)? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Platform-specific build
case "$PLATFORM" in
    Linux)
        echo "Building for Linux..."
        if command -v zig &> /dev/null; then
            # Use zig for better cross-compilation
            maturin build --release --zig \
                --strip \
                --target x86_64-unknown-linux-gnu \
                --target aarch64-unknown-linux-gnu \
                --out dist \
                --compatibility manylinux2014
        else
            # Fallback to native build
            maturin build --release --strip --out dist \
                --compatibility manylinux2014
        fi
        ;;
    
    Darwin)
        echo "Building for macOS..."
        
        # Check if building universal2 or native only
        if [[ "${UNIVERSAL:-}" == "1" ]]; then
            echo "Building universal2 wheel (requires: rustup target add aarch64-apple-darwin)"
            # Check if aarch64 target is installed
            if ! rustup target list | grep -q "aarch64-apple-darwin (installed)"; then
                echo "⚠️  aarch64-apple-darwin target not found"
                echo "   Run: rustup target add aarch64-apple-darwin"
                exit 1
            fi
            maturin build --release --strip \
                --target universal2-apple-darwin \
                --out dist
        else
            echo "Building native wheel for $ARCH (set UNIVERSAL=1 for universal2)"
            maturin build --release --strip --out dist
        fi
        ;;
    
    MINGW*|MSYS*|CYGWIN*)
        echo "Building for Windows..."
        maturin build --release --strip --out dist
        ;;
    
    *)
        echo "Unknown platform: $PLATFORM"
        echo "Attempting generic build..."
        maturin build --release --strip --out dist
        ;;
esac

echo ""
echo "✅ Build complete. Wheels are in dist/"
ls -lh dist/