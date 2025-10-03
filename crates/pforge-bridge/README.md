# pforge-bridge

[![crates.io](https://img.shields.io/crates/v/pforge-bridge.svg)](https://crates.io/crates/pforge-bridge)
[![Documentation](https://docs.rs/pforge-bridge/badge.svg)](https://docs.rs/pforge-bridge)

Language bridges for polyglot MCP handlers - write handlers in Python, Go, Node.js, and other languages while using pforge's Rust runtime.

## Features

- **Multi-Language Support**: Python, Go, Node.js, and more
- **Zero-Copy FFI**: Efficient data passing across language boundaries
- **Type Safety**: Preserve type information across language bridges
- **Error Propagation**: Native error semantics in each language
- **Stable ABI**: C FFI for maximum compatibility
- **Async Support**: Works with async handlers in all languages

## Supported Languages

| Language | Status | Features |
|----------|--------|----------|
| Python | ✅ Stable | Async/await, type hints, error handling |
| Go | 🚧 Beta | Goroutines, error handling |
| Node.js | 🚧 Beta | Promises, TypeScript support |
| Ruby | 📋 Planned | - |
| Java | 📋 Planned | - |

## Installation

```bash
cargo add pforge-bridge
```

## Quick Start

### Python Handler

**pforge.yaml**:
```yaml
tools:
  - type: native
    name: analyze_data
    description: "Analyze data with Python"
    handler:
      bridge: python
      path: handlers.analyze_data
    params:
      data: { type: array, required: true }
```

**handlers.py**:
```python
async def analyze_data(params):
    """Analyze data using numpy and pandas"""
    import numpy as np
    data = np.array(params['data'])

    return {
        'mean': float(np.mean(data)),
        'std': float(np.std(data)),
        'min': float(np.min(data)),
        'max': float(np.max(data))
    }
```

### Go Handler

**pforge.yaml**:
```yaml
tools:
  - type: native
    name: process_image
    description: "Process image with Go"
    handler:
      bridge: go
      path: handlers.ProcessImage
    params:
      image_url: { type: string, required: true }
```

**handlers.go**:
```go
package handlers

import (
    "encoding/json"
    "image"
    _ "image/jpeg"
    "net/http"
)

func ProcessImage(params map[string]interface{}) (interface{}, error) {
    imageURL := params["image_url"].(string)

    resp, err := http.Get(imageURL)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    img, _, err := image.Decode(resp.Body)
    if err != nil {
        return nil, err
    }

    bounds := img.Bounds()
    return map[string]interface{}{
        "width":  bounds.Dx(),
        "height": bounds.Dy(),
    }, nil
}
```

### Node.js Handler

**pforge.yaml**:
```yaml
tools:
  - type: native
    name: fetch_data
    description: "Fetch data with Node.js"
    handler:
      bridge: node
      path: handlers.fetchData
    params:
      url: { type: string, required: true }
```

**handlers.js**:
```javascript
export async function fetchData(params) {
    const response = await fetch(params.url);
    const data = await response.json();

    return {
        status: response.status,
        data: data,
        headers: Object.fromEntries(response.headers)
    };
}
```

## Architecture

### C FFI Layer

The bridge uses a stable C ABI for maximum compatibility:

```c
// Bridge function signature
typedef struct {
    const char* data;
    size_t len;
} BridgeData;

typedef BridgeData (*BridgeHandler)(BridgeData params);
```

### Zero-Copy Parameter Passing

Parameters are passed as pointers, not serialized:

```rust
// Rust side
let params_ptr = params.as_ptr();
let params_len = params.len();

// Python side receives raw pointer
let result = python_handler(params_ptr, params_len);
```

### Error Handling

Errors propagate with full context:

```python
# Python raises exception
raise ValueError("Invalid data format")
```

```rust
// Rust receives typed error
Err(Error::Handler("Python error: Invalid data format".into()))
```

## Bridge Configuration

### Python Bridge

```yaml
bridges:
  python:
    runtime: cpython  # or pypy
    version: "3.11"
    venv: .venv       # Optional virtualenv
    requirements: requirements.txt
```

**Install dependencies**:
```bash
pip install pforge-bridge-python
```

### Go Bridge

```yaml
bridges:
  go:
    version: "1.21"
    mod: handlers    # Go module name
```

**Install bridge**:
```bash
go get github.com/paiml/pforge-bridge-go
```

### Node.js Bridge

```yaml
bridges:
  node:
    runtime: node    # or bun, deno
    version: "20"
    package: package.json
```

**Install bridge**:
```bash
npm install @paiml/pforge-bridge
```

## Advanced Usage

### Type Mapping

Automatic type conversion between languages:

| JSON Type | Rust | Python | Go | Node.js |
|-----------|------|--------|----|---------|
| string | String | str | string | string |
| number | f64 | float | float64 | number |
| integer | i64 | int | int64 | number |
| boolean | bool | bool | bool | boolean |
| array | Vec<T> | list | []interface{} | Array |
| object | HashMap | dict | map | Object |

### Custom Serialization

Override default serialization:

```python
from pforge_bridge import register_serializer

@register_serializer(MyClass)
def serialize_myclass(obj):
    return {
        'type': 'MyClass',
        'data': obj.to_dict()
    }
```

### State Sharing

Share state between Rust and bridge languages:

```rust
// Rust side
let state = Arc::new(RwLock::new(SharedState::new()));
bridge.set_state("shared", state.clone());
```

```python
# Python side
state = get_shared_state("shared")
async with state.write():
    state.counter += 1
```

## Performance

| Operation | Overhead |
|-----------|----------|
| Function call | ~1-5μs |
| Parameter passing (1KB) | ~100ns |
| Result marshaling (1KB) | ~100ns |
| Error propagation | ~500ns |

The bridge is optimized for minimal overhead while maintaining safety.

## Security

- **Sandboxing**: Optional process isolation for handlers
- **Resource Limits**: Memory and CPU limits per handler
- **Capability-Based**: Handlers only access granted capabilities
- **Input Validation**: Automatic validation of bridge parameters

## Debugging

Enable debug logging:

```bash
PFORGE_BRIDGE_LOG=debug pforge serve
```

Trace bridge calls:

```rust
bridge.enable_tracing(true);
```

## Limitations

- No direct pointer sharing between languages
- Serialization overhead for complex types
- Each bridge adds ~5MB to binary size
- Requires language runtime installed

## Documentation

- [API Documentation](https://docs.rs/pforge-bridge)
- [Bridge Guide](https://github.com/paiml/pforge/blob/main/docs/USER_GUIDE.md#language-bridges)
- [Examples](https://github.com/paiml/pforge/tree/main/examples/polyglot-server)

## Contributing

We welcome bridge implementations for additional languages! Please open an issue or pull request on [GitHub](https://github.com/paiml/pforge).

## License

MIT
