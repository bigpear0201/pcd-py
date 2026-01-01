# pcd-py: Python 高性能 PCD I/O 库

`pcd-py` 是一个用于读写 PCD (Point Cloud Data) 文件的极速 Python 库，基于 Rust (`pcd-rs`) 核心实现。它与 **NumPy** 无缝集成，实现高效的数据处理。

## 主要特性
- **极速 I/O**: 利用 Rust 实现多线程二进制和压缩格式的 PCD 处理。
- **NumPy 集成**: 零拷贝机制，直接将 PCD 字段作为 NumPy 数组读写。
- **全格式支持**: 支持 `ASCII`、`Binary` 和 `Binary Compressed` 格式。
- **元数据访问**: 轻松获取 PCD 头部信息（版本、宽高、视点等）。

## 安装指南

```bash
# 需要安装 maturin 以进行源码构建
pip install maturin numpy
cd pcd-py
maturin develop --release
```

## 快速开始

```python
import pcd_py
import numpy as np

# 1. 读取 PCD 文件
meta, data = pcd_py.read_pcd("example.pcd")

print(f"Points: {meta.points}")
print(f"Fields: {data.keys()}")

x = data["x"]  # numpy array (f32)
intensity = data["intensity"]  # numpy array (f32)

# 2. 写入 PCD 文件
new_data = {
    "x": np.array([1.0, 2.0, 3.0], dtype=np.float32),
    "y": np.array([0.0, 0.0, 0.0], dtype=np.float32),
    "z": np.array([5.0, 5.0, 5.0], dtype=np.float32),
    "id": np.array([1, 2, 3], dtype=np.uint32),
}

# 支持 binary 或 binary_compressed 格式
pcd_py.write_pcd("output.pcd", new_data, format="binary_compressed")
```

## 性能表现

对于 **1,000,000 点** 的点云数据 (XYZIRT 格式):
- **读取 (Binary)**: **~12 ms** (采用零拷贝 Mmap 技术)
- **写入 (Binary)**: **~135 ms**

> [!NOTE]
> 得益于零拷贝内存映射 (Zero-Copy Mmap) 和并行解码技术，**读取速度提高了 12 倍**，性能现已**媲美原生 Rust** (100万点读取仅需 12ms)。

## 开源协议
Apache-2.0
