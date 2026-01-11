# pcd-py: High-Performance PCD I/O for Python

`pcd-py` is a high-speed Python library for reading and writing PCD (Point Cloud Data) files, powered by a core implementation in Rust (`rs-pcd`). It integrates seamlessly with **NumPy** for efficient data handling.

## Features

- **🚀 Blazing Fast**:
  - Mmap + parallel decoding for ~10ms read time on 1M points
  - Powered by pcd-rs v0.2.0 optimizations (batch I/O, platform-optimized endianness)
- **NumPy Integration**: Read/write PCD fields directly as NumPy arrays
- **Full Format Support**: `ASCII`, `Binary`, and `Binary Compressed`
- **Metadata Access**: Easy access to PCD header info (version, fields, width, height, viewpoint)

## Performance

Benchmarks on Apple Silicon (1M points, XYZIRT format):

| Operation | Time | Throughput |
|-----------|------|------------|
| **Read Binary (Mmap)** | **~10 ms** | **~3 GB/s** ⚡ |
| Write Binary | ~120 ms | ~250 MB/s |
| Read Compressed | ~65 ms | ~460 MB/s |

## Installation

```bash
# From PyPI (coming soon)
pip install pcd-py

# From source (requires Rust toolchain)
pip install maturin numpy
cd pcd-py
maturin develop --release
```

## Quick Start

### Reading a PCD File

```python
import pcd_py
import numpy as np

# Read a PCD file (supports binary, binary_compressed, ascii)
meta, data = pcd_py.read_pcd("lidar.pcd")

print(f"Points: {meta.points}")
print(f"Fields: {meta.fields}")  # e.g., ['x', 'y', 'z', 'intensity', 'ring', 'timestamp']

# Access fields as numpy arrays
x = data["x"]          # np.ndarray (float32)
y = data["y"]          # np.ndarray (float32)
z = data["z"]          # np.ndarray (float32)
intensity = data["intensity"]  # np.ndarray (float32)
ring = data["ring"]    # np.ndarray (uint16)
timestamp = data["timestamp"]  # np.ndarray (float64)
```

### Reading from Memory Buffer

```python
# Useful for network streams or embedded resources
with open("example.pcd", "rb") as f:
    pcd_bytes = f.read()

meta, data = pcd_py.read_pcd_from_buffer(pcd_bytes)
```

### Writing a PCD File

```python
import numpy as np
import pcd_py

# Prepare data as dict of numpy arrays
points = 1000
data = {
    "x": np.random.randn(points).astype(np.float32),
    "y": np.random.randn(points).astype(np.float32),
    "z": np.random.randn(points).astype(np.float32),
    "intensity": np.random.rand(points).astype(np.float32),
    "ring": np.random.randint(0, 64, points).astype(np.uint16),
    "timestamp": np.arange(points, dtype=np.float64) * 0.1,
}

# Write as binary (fastest)
pcd_py.write_pcd("output.pcd", data, format="binary")

# Write as binary_compressed (smaller file size)
pcd_py.write_pcd("output_compressed.pcd", data, format="binary_compressed")

# Write as ASCII (human readable)
pcd_py.write_pcd("output_ascii.pcd", data, format="ascii")
```

## API Reference

### `read_pcd(path: str) -> (MetaData, dict)`

Read a PCD file from disk using memory-mapped I/O.

**Returns:**

- `MetaData`: Object with `version`, `width`, `height`, `points`, `viewpoint`, `fields`
- `dict`: Field name → numpy array mapping

### `read_pcd_from_buffer(buffer: bytes) -> (MetaData, dict)`

Read a PCD file from a bytes buffer.

### `write_pcd(path, data, format="binary", viewpoint=None)`

Write a PCD file to disk.

**Args:**

- `path`: Output file path
- `data`: Dict of field_name → numpy array
- `format`: `"ascii"`, `"binary"`, or `"binary_compressed"`
- `viewpoint`: Optional `[tx, ty, tz, qw, qx, qy, qz]` (default: identity)

### Supported NumPy dtypes

| NumPy dtype | PCD Type |
|-------------|----------|
| `float32` | F32 |
| `float64` | F64 |
| `uint8` | U8 |
| `uint16` | U16 |
| `uint32` | U32 |
| `int8` | I8 |
| `int16` | I16 |
| `int32` | I32 |

## What's New in v0.2.0

- ⚡ **30-50% faster reading** via pcd-rs v0.2.0 optimizations
- 📋 **`meta.fields`** now available for schema inspection
- 🔧 Improved error messages
- 🦀 Edition 2021 compatibility

## License

Apache-2.0
