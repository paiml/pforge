#!/usr/bin/env python3
"""
Example usage of pforge Python bridge
"""

from pforge_bridge import PforgeBridge

def main():
    bridge = PforgeBridge()

    print("=" * 60)
    print(f"pforge Python Bridge Example")
    print(f"Version: {bridge.version()}")
    print("=" * 60)

    # Execute handler with JSON input
    input_data = {
        "operation": "greet",
        "name": "Python",
        "count": 3
    }

    print(f"\nInput: {input_data}")

    result = bridge.execute_handler("python_handler", input_data)

    print(f"Output: {result}")
    print(f"Handler: {result.get('handler')}")
    print(f"Status: {result.get('status')}")

if __name__ == "__main__":
    main()
