"""
Python bridge for pforge FFI

This module provides a Pythonic interface to pforge's C FFI bridge.
"""

import ctypes
import json
import os
from pathlib import Path
from typing import Any, Dict, Optional

# Locate the shared library
def find_library():
    """Find the pforge_bridge shared library"""
    # Try common locations
    possible_paths = [
        Path("../../target/debug/libpforge_bridge.so"),  # Linux debug
        Path("../../target/release/libpforge_bridge.so"),  # Linux release
        Path("../../target/debug/libpforge_bridge.dylib"),  # macOS debug
        Path("../../target/release/libpforge_bridge.dylib"),  # macOS release
        Path("../../target/debug/pforge_bridge.dll"),  # Windows debug
        Path("../../target/release/pforge_bridge.dll"),  # Windows release
    ]

    script_dir = Path(__file__).parent
    for path in possible_paths:
        full_path = script_dir / path
        if full_path.exists():
            return str(full_path.resolve())

    raise FileNotFoundError(
        "Could not find pforge_bridge library. "
        "Run 'cargo build -p pforge-bridge' first."
    )

# Load the library
lib_path = find_library()
lib = ctypes.CDLL(lib_path)

# Define C structures
class FfiResult(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int),
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("data_len", ctypes.c_size_t),
        ("error", ctypes.c_char_p),
    ]

# Define C function signatures
lib.pforge_version.restype = ctypes.c_char_p
lib.pforge_version.argtypes = []

lib.pforge_execute_handler.restype = FfiResult
lib.pforge_execute_handler.argtypes = [
    ctypes.c_char_p,  # handler_name
    ctypes.POINTER(ctypes.c_ubyte),  # input_json
    ctypes.c_size_t,  # input_len
]

lib.pforge_free_result.restype = None
lib.pforge_free_result.argtypes = [FfiResult]


class PforgeBridge:
    """Python interface to pforge FFI bridge"""

    @staticmethod
    def version() -> str:
        """Get pforge version"""
        return lib.pforge_version().decode('utf-8')

    @staticmethod
    def execute_handler(handler_name: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute a pforge handler

        Args:
            handler_name: Name of the handler to execute
            input_data: Input data as a dictionary

        Returns:
            Handler result as a dictionary

        Raises:
            RuntimeError: If the handler execution fails
        """
        # Serialize input to JSON
        input_json = json.dumps(input_data).encode('utf-8')
        input_array = (ctypes.c_ubyte * len(input_json)).from_buffer_copy(input_json)

        # Call FFI
        result = lib.pforge_execute_handler(
            handler_name.encode('utf-8'),
            input_array,
            len(input_json)
        )

        try:
            # Check for errors
            if result.code != 0:
                error_msg = result.error.decode('utf-8') if result.error else "Unknown error"
                raise RuntimeError(f"Handler execution failed (code {result.code}): {error_msg}")

            # Extract result data
            if result.data and result.data_len > 0:
                data_bytes = bytes(result.data[:result.data_len])
                return json.loads(data_bytes)
            else:
                return {}

        finally:
            # Always free the result
            lib.pforge_free_result(result)


# Example usage
if __name__ == "__main__":
    bridge = PforgeBridge()

    print(f"pforge version: {bridge.version()}")

    result = bridge.execute_handler("test_handler", {"value": 42})
    print(f"Result: {json.dumps(result, indent=2)}")
