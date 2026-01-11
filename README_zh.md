# pcd-py: Python 高性能 PCD 读写库

`pcd-py` 是一个基于 Rust (`rs-pcd`) 内核的极速 Python 点云库，与 **NumPy** 无缝集成。

## 核心特性

- **🚀 极速性能**:
  - Mmap + 并行解码，100万点读取仅需 ~10ms
  - 基于 pcd-rs v0.2.0 优化（批量 I/O、平台优化字节序）
- **NumPy 集成**: 零拷贝直接读写 NumPy 数组
- **全格式支持**: `ASCII`、`Binary`、`Binary Compressed`
- **元数据访问**: 轻松获取头部信息（版本、字段、宽高、视点）

## 性能表现

Apple Silicon 测试（100万点，XYZIRT 格式）：

| 操作 | 耗时 | 吞吐量 |
|------|------|--------|
| **读取 Binary (Mmap)** | **~10 ms** | **~3 GB/s** ⚡ |
| 写入 Binary | ~120 ms | ~250 MB/s |
| 读取 Compressed | ~65 ms | ~460 MB/s |

## 安装

```bash
# 从 PyPI 安装（即将发布）
pip install pcd-py

# 从源码安装（需要 Rust 工具链）
pip install maturin numpy
cd pcd-py
maturin develop --release
```

## 快速开始

### 读取 PCD 文件

```python
import pcd_py
import numpy as np

# 读取 PCD 文件（支持 binary, binary_compressed, ascii）
meta, data = pcd_py.read_pcd("lidar.pcd")

print(f"点数: {meta.points}")
print(f"字段: {meta.fields}")  # 例如 ['x', 'y', 'z', 'intensity', 'ring', 'timestamp']

# 以 numpy 数组形式访问字段
x = data["x"]          # np.ndarray (float32)
y = data["y"]          # np.ndarray (float32)
z = data["z"]          # np.ndarray (float32)
intensity = data["intensity"]  # np.ndarray (float32)
ring = data["ring"]    # np.ndarray (uint16)
timestamp = data["timestamp"]  # np.ndarray (float64)
```

### 从内存缓冲区读取

```python
# 适用于网络流或嵌入式资源
with open("example.pcd", "rb") as f:
    pcd_bytes = f.read()

meta, data = pcd_py.read_pcd_from_buffer(pcd_bytes)
```

### 写入 PCD 文件

```python
import numpy as np
import pcd_py

# 准备数据（numpy 数组字典）
points = 1000
data = {
    "x": np.random.randn(points).astype(np.float32),
    "y": np.random.randn(points).astype(np.float32),
    "z": np.random.randn(points).astype(np.float32),
    "intensity": np.random.rand(points).astype(np.float32),
    "ring": np.random.randint(0, 64, points).astype(np.uint16),
    "timestamp": np.arange(points, dtype=np.float64) * 0.1,
}

# 写入 binary 格式（最快）
pcd_py.write_pcd("output.pcd", data, format="binary")

# 写入 binary_compressed 格式（文件更小）
pcd_py.write_pcd("output_compressed.pcd", data, format="binary_compressed")

# 写入 ASCII 格式（人类可读）
pcd_py.write_pcd("output_ascii.pcd", data, format="ascii")
```

## API 参考

### `read_pcd(path: str) -> (MetaData, dict)`

使用内存映射读取 PCD 文件。

**返回:**

- `MetaData`: 包含 `version`, `width`, `height`, `points`, `viewpoint`, `fields`
- `dict`: 字段名 → numpy 数组映射

### `read_pcd_from_buffer(buffer: bytes) -> (MetaData, dict)`

从字节缓冲区读取 PCD 文件。

### `write_pcd(path, data, format="binary", viewpoint=None)`

写入 PCD 文件。

**参数:**

- `path`: 输出文件路径
- `data`: 字段名 → numpy 数组的字典
- `format`: `"ascii"`, `"binary"`, 或 `"binary_compressed"`
- `viewpoint`: 可选的 `[tx, ty, tz, qw, qx, qy, qz]`（默认: 单位变换）

### 支持的 NumPy 类型

| NumPy 类型 | PCD 类型 |
|------------|----------|
| `float32` | F32 |
| `float64` | F64 |
| `uint8` | U8 |
| `uint16` | U16 |
| `uint32` | U32 |
| `int8` | I8 |
| `int16` | I16 |
| `int32` | I32 |

## v0.2.0 新特性

- ⚡ **读取速度提升 30-50%**，基于 pcd-rs v0.2.0 优化
- 📋 **`meta.fields`** 现可获取字段列表
- 🔧 改进的错误信息
- 🦀 Edition 2021 兼容

## 开源协议

Apache-2.0
