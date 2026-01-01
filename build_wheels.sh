#!/bin/bash
set -e

# Define project root (parent of pcd-py)
PROJECT_ROOT=$(cd .. && pwd)
echo "Project Root: $PROJECT_ROOT"

# Ensure output directory exists
mkdir -p dist

# Run maturin build inside Docker (manylinux2014)
# We mount the entire workspace ($PROJECT_ROOT) to /io so that pcd-rs (sibling of pcd-py) is accessible.
# We set the working directory to /io/pcd-py.

echo "Building manylinux wheels..."
docker run --rm -v "$PROJECT_ROOT":/io \
    -w /io/pcd-py \
    ghcr.io/pyo3/maturin build --release --manylinux 2014 --strip

echo "Build complete. Wheels are in target/wheels/"
