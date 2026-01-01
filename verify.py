import pcd_py
import numpy as np
import os

def test_roundtrip():
    print("--- Testing Round-trip ---")
    points = 100
    data = {
        "x": np.linspace(0, 10, points).astype(np.float32),
        "y": np.linspace(0, 20, points).astype(np.float32),
        "z": np.linspace(0, 30, points).astype(np.float32),
        "intensity": np.linspace(0, 1, points).astype(np.float32),
        "id": np.arange(points).astype(np.uint32),
    }

    filename = "test_py.pcd"
    
    print(f"Writing {filename} (binary_compressed)...")
    pcd_py.write_pcd(filename, data, format="binary_compressed")

    print(f"Reading {filename}...")
    meta, read_data = pcd_py.read_pcd(filename)

    print(f"MetaData: points={meta.points}, width={meta.width}, height={meta.height}")
    
    # Verify values
    for name, original in data.items():
        read_val = read_data[name]
        if not np.allclose(original, read_val):
            print(f"Mismatch in field: {name}")
            return False
        else:
            print(f"Field {name} verified.")

    print("Success!")
    return True

if __name__ == "__main__":
    if test_roundtrip():
        print("\nAll tests passed!")
    else:
        print("\nTests failed!")
