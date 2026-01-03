#!/bin/bash
set -e

# Define project root
PROJECT_ROOT=$(cd .. && pwd)
echo "Project Root: $PROJECT_ROOT"

# Ensure output directory exists
mkdir -p dist

# Check for required tools
if ! command -v maturin &> /dev/null; then
    echo "Error: maturin is not installed. Please install it with 'pip install maturin'."
    exit 1
fi

if ! command -v zig &> /dev/null; then
    echo "Warning: zig is not installed. Zig is recommended for easier cross-compilation."
    echo "Falling back to default maturin build (which may require docker if not on linux)."
    USE_ZIG=""
else
    echo "Found zig, enabling cross-compilation support."
    USE_ZIG="--zig"
fi

echo "Building wheels..."

# Build with abi3 support (enabled in Cargo.toml via pyo3/abi3-py38)
# This creates a single wheel compatible with Python 3.8+
# We target x86_64-unknown-linux-gnu (manylinux) using zig for max compatibility
# You can add --target aarch64-unknown-linux-gnu for ARM support

# Note: We are using --find-interpreter to ensure we find a suitable python if needed,
# but with abi3 it might pick one and build a generic wheel.
maturin build --release $USE_ZIG \
    --strip \
    --target x86_64-unknown-linux-gnu \
    --out dist \
    --compatibility manylinux_2_28

echo "Build complete. Wheels are in dist/"
ls -lh dist/
