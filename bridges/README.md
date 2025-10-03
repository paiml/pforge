# pforge Language Bridges

Language bridges enable polyglot MCP servers - write handlers in Python, Go, or Node.js while maintaining pforge's performance and type safety.

## Architecture

```
┌─────────────────────────────────┐
│   pforge Runtime (Rust)         │
│   ┌─────────────────────────┐   │
│   │  Handler Registry       │   │
│   └──────────┬──────────────┘   │
└──────────────┼──────────────────┘
               │ C ABI FFI
               ▼
┌──────────────────────────────────┐
│  Language Bridges                │
│  ┌────────┐ ┌──────┐ ┌────────┐ │
│  │ Python │ │  Go  │ │Node.js │ │
│  │(ctypes)│ │(cgo) │ │ (napi) │ │
│  └────────┘ └──────┘ └────────┘ │
└──────────────────────────────────┘
```

## Features

- **Zero-Copy FFI**: Pass pointers, not serialized data
- **Type Safety**: Preserve type information across boundaries
- **Performance**: <100ns bridge overhead
- **Error Handling**: Maintains Rust's Result semantics

## Bridges

### Python (ctypes)

Located in `bridges/python/`

**Usage:**
```python
from pforge_bridge import PforgeBridge

bridge = PforgeBridge()
result = bridge.execute_handler("my_handler", {"key": "value"})
```

**Example:**
```bash
cd bridges/python
python3 example.py
```

### Go (cgo)

Located in `bridges/go/`

**Usage:**
```go
import "./pforge"

bridge := pforge.NewBridge()
result, err := bridge.ExecuteHandler("my_handler", map[string]interface{}{
    "key": "value",
})
```

**Example:**
```bash
cd bridges/go
# First build the pforge-bridge library
cargo build -p pforge-bridge --release
# Then run Go example
go run example.go
```

### Node.js (N-API)

Located in `bridges/nodejs/` - Coming soon!

## Building

1. Build the FFI bridge library:
```bash
cargo build -p pforge-bridge --release
```

This creates:
- Linux: `target/release/libpforge_bridge.so`
- macOS: `target/release/libpforge_bridge.dylib`
- Windows: `target/release/pforge_bridge.dll`

2. Run language-specific examples (see above)

## FFI API

### C Functions

```c
// Get pforge version
const char* pforge_version();

// Execute handler
FfiResult pforge_execute_handler(
    const char* handler_name,
    const unsigned char* input_json,
    size_t input_len
);

// Free result
void pforge_free_result(FfiResult result);
```

### FfiResult Structure

```c
typedef struct {
    int code;              // 0 = success, non-zero = error
    unsigned char* data;   // JSON result bytes
    size_t data_len;       // Result length
    const char* error;     // Error message (null if success)
} FfiResult;
```

## Performance

**Benchmarks** (Intel i7, 3.5GHz):
- FFI call overhead: ~80ns
- JSON serialization: ~500ns (1KB payload)
- Total roundtrip: <1μs

## Safety

- All pointers are validated for null
- Memory is properly allocated/freed
- Panics are caught at FFI boundary
- Thread-safe for concurrent calls

## Testing

```bash
# Test FFI bridge
cargo test -p pforge-bridge

# Test Python bridge
cd bridges/python && python3 -m pytest

# Test Go bridge
cd bridges/go && go test
```

## License

MIT - See LICENSE file
