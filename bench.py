import pcd_py
import numpy as np
import time
import os

def benchmark_1m():
    print("--- 1 Million Points Benchmark ---")
    points = 1_000_000
    data = {
        "x": np.linspace(0, 100, points).astype(np.float32),
        "y": np.linspace(0, 100, points).astype(np.float32),
        "z": np.linspace(-20, 20, points).astype(np.float32),
        "intensity": np.random.rand(points).astype(np.float32),
        "ring": np.random.randint(0, 64, points).astype(np.uint16),
        "timestamp": np.linspace(1700000000, 1700000100, points).astype(np.float64),
    }

    filename = "bench_1m.pcd"
    
    # Write Binary
    start = time.time()
    pcd_py.write_pcd(filename, data, format="binary")
    end = time.time()
    print(f"Write Binary (1M pts): {(end - start)*1000:.2f} ms")

    # Read Binary
    start = time.time()
    meta, read_data = pcd_py.read_pcd(filename)
    end = time.time()
    print(f"Read Binary (1M pts): {(end - start)*1000:.2f} ms")

    # Write Compressed
    start = time.time()
    pcd_py.write_pcd(filename, data, format="binary_compressed")
    end = time.time()
    print(f"Write Compressed (1M pts): {(end - start)*1000:.2f} ms")

    # Read Compressed
    start = time.time()
    meta, read_data = pcd_py.read_pcd(filename)
    end = time.time()
    print(f"Read Compressed (1M pts): {(end - start)*1000:.2f} ms")

    if os.path.exists(filename):
        os.remove(filename)

if __name__ == "__main__":
    benchmark_1m()
