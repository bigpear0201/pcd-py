import pcd_py
import numpy as np
import os

def test_buffer_read():
    print("--- Testing Buffer Reading ---")
    filename = "buffer_test.pcd"
    
    # Generate data
    points = 1000
    data = {
        "x": np.linspace(0, 10, points).astype(np.float32),
        "y": np.linspace(0, 20, points).astype(np.float32),
        "z": np.linspace(0, 30, points).astype(np.float32),
    }

    # Write to file first
    pcd_py.write_pcd(filename, data, format="binary")

    # Read file into bytes
    with open(filename, "rb") as f:
        file_bytes = f.read()
    
    # Read using new API
    meta, read_data = pcd_py.read_pcd_from_buffer(file_bytes)
    
    print(f"Read {meta.points} points from buffer.")
    
    # Validation
    if meta.points != points:
        print("FAILED: Point count mismatch")
        return False

    if not np.allclose(data["x"], read_data["x"]):
        print("FAILED: Data mismatch")
        return False
        
    print("Success: Read from buffer verified.")
    if os.path.exists(filename):
        os.remove(filename)
    return True

if __name__ == "__main__":
    if not test_buffer_read():
        exit(1)
