# pcd-py: High-Performance PCD I/O for Python

`pcd-py` is a high-speed Python library for reading and writing PCD (Point Cloud Data) files, powered by a core implementation in Rust (`pcd-rs`). It integrates seamlessly with **NumPy** for efficient data handling.

## Features
- **Fast I/O**: Leverages Rust for multi-threaded binary and compressed PCD handling.
- **NumPy Integration**: Reads/writes PCD fields directly as NumPy arrays.
- **Full Format Support**: Supports `ASCII`, `Binary`, and `Binary Compressed` formats.
- **Metadata Access**: Easy access to PCD header information (version, width, height, viewpoint).

## Installation

```bash
# Requires maturin to build from source
pip install maturin numpy
cd pcd-py
maturin develop --release
```

## Quick Start

```python
import pcd_py
import numpy as np

# 1. Read a PCD file
meta, data = pcd_py.read_pcd("example.pcd")

print(f"Points: {meta.points}")
print(f"Fields: {data.keys()}")

x = data["x"]  # numpy array (f32)
intensity = data["intensity"]  # numpy array (f32)

# 2. Read from memory (e.g., from network)
with open("example.pcd", "rb") as f:
    data_bytes = f.read()

meta_buf, data_buf = pcd_py.read_pcd_from_buffer(data_bytes)

# 3. Write a PCD file
new_data = {
    "x": np.array([1.0, 2.0, 3.0], dtype=np.float32),
    "y": np.array([0.0, 0.0, 0.0], dtype=np.float32),
    "z": np.array([5.0, 5.0, 5.0], dtype=np.float32),
    "id": np.array([1, 2, 3], dtype=np.uint32),
}

pcd_py.write_pcd("output.pcd", new_data, format="binary_compressed")
```

## Performance

For a point cloud with **1,000,000 points** (XYZIRT schema):
- **Read Binary**: **~12 ms** (Zero-Copy Mmap)
- **Write Binary**: **~135 ms**

> [!NOTE]
> Performance is now **equivalent to native Rust**, reading 1 million points in under 12ms.

## License
Apache-2.0
