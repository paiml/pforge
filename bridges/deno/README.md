# pforge Deno/TypeScript Bridge

> Build MCP (Model Context Protocol) servers with TypeScript and Deno

[![Tests](https://img.shields.io/badge/tests-74%20passing-brightgreen)](./tests)
[![Quality](https://img.shields.io/badge/SATD-0-brightgreen)](./docs)
[![Complexity](https://img.shields.io/badge/complexity-%E2%89%A420-brightgreen)](./docs)
[![Schema](https://img.shields.io/badge/schema-validation-blue)](./docs/schema-validation.md)

A high-performance, type-safe bridge for building Model Context Protocol servers
using TypeScript and Deno. Seamlessly integrates with the Rust pforge runtime
via FFI while providing an ergonomic TypeScript API.

## 🚀 Quick Start

```typescript
import { PforgeBridge } from "https://raw.githubusercontent.com/your-org/pforge/main/bridges/deno/bridge.ts";

// Create bridge
const bridge = new PforgeBridge();

// Register a tool
bridge.register({
  name: "greet",
  description: "Greet a user by name",
  handler: (input: { name: string }) => ({
    success: true,
    data: { message: `Hello, ${input.name}! 👋` },
  }),
});

// Execute the tool
const result = await bridge.execute("greet", { name: "Alice" });
console.log(result);
// => { success: true, data: { message: "Hello, Alice! 👋" } }

// Cleanup
bridge.close();
```

## 📦 Installation

### Prerequisites

1. **Deno** (v2.0+): [Install Deno](https://deno.land/#installation)
2. **Rust pforge library**: Build the FFI bridge
   ```bash
   cd ../../  # Navigate to pforge root
   cargo build -p pforge-bridge --release
   ```

### Import in Your Project

```typescript
import { PforgeBridge } from "https://raw.githubusercontent.com/your-org/pforge/main/bridges/deno/bridge.ts";
```

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│  Your TypeScript MCP Server         │
│  ┌───────────────────────────────┐  │
│  │   PforgeBridge                │  │
│  │   • register() - Add tools    │  │
│  │   • execute() - Run tools     │  │
│  │   • list() - List tools       │  │
│  └────────┬──────────┬───────────┘  │
│           │          │               │
│  ┌────────▼──────┐  │               │
│  │ TypeScript    │  │               │
│  │ Handlers      │  │               │
│  └───────────────┘  │               │
│                     │               │
│           ┌─────────▼──────────┐    │
│           │ FFI Bridge (Deno)  │    │
│           └─────────┬──────────┘    │
└─────────────────────┼───────────────┘
                      │
              ┌───────▼────────┐
              │ Rust pforge    │
              │ (C ABI / FFI)  │
              └────────────────┘
```

**Key Components:**

- **PforgeBridge**: High-level API for tool registration and execution
- **HandlerRegistry**: O(1) lookup, type-safe handler management
- **FfiBridge**: Low-level FFI bindings to Rust pforge runtime

## 🎯 Features

### Type Safety

- **Generic Handlers**: Full TypeScript type inference
- **Strict Type Checking**: Compile-time safety
- **Runtime Schema Validation**: Built-in JSON schema validation ([docs](./docs/schema-validation.md))
- **Result Types**:
  `{ success: true, data: T } | { success: false, error: string }`

### Performance

- **O(1) Handler Lookup**: HashMap-based registry
- **Zero-Copy JSON**: Minimal serialization overhead
- **9.5µs Execution**: Per-call overhead (reused bridge)
- **Memory Safe**: Validated with 1000+ property-based tests

### Developer Experience

- **Simple API**: Just `register()` and `execute()`
- **Async/Sync Support**: Both handler types work
- **Clear Errors**: Descriptive error messages
- **Working Examples**: Learn by example

## 📚 API Reference

### `PforgeBridge`

Main class for building MCP servers.

#### Constructor

```typescript
const bridge = new PforgeBridge();
```

#### Methods

##### `register<TInput, TOutput>(handler: HandlerDef)`

Register a new tool handler.

```typescript
bridge.register({
  name: "calculate",
  description: "Perform calculations",
  handler: (input: { a: number; b: number; op: string }) => {
    const operations = {
      add: input.a + input.b,
      subtract: input.a - input.b,
      multiply: input.a * input.b,
    };

    return {
      success: true,
      data: { result: operations[input.op] },
    };
  },
  timeoutMs: 5000, // Optional timeout (default: 30000)
});
```

**Parameters:**

- `name`: Unique tool name
- `description`: Tool description
- `handler`: Function that processes input and returns result
- `timeoutMs`: Optional timeout in milliseconds

##### `execute<TOutput>(name: string, input: unknown)`

Execute a tool by name.

```typescript
const result = await bridge.execute("calculate", {
  a: 10,
  b: 5,
  op: "add",
});

if (result.success) {
  console.log(result.data); // => { result: 15 }
} else {
  console.error(result.error);
}
```

**Returns:** `Promise<HandlerResult<TOutput>>`

##### `list(): string[]`

Get list of registered tool names.

```typescript
const tools = bridge.list();
console.log(tools); // => ["calculate", "greet", "fetch_data"]
```

##### `has(name: string): boolean`

Check if a tool is registered.

```typescript
if (bridge.has("calculate")) {
  // Tool exists
}
```

##### `count(): number`

Get number of registered tools.

```typescript
console.log(bridge.count()); // => 3
```

##### `version(): string`

Get pforge version.

```typescript
console.log(bridge.version()); // => "0.1.2"
```

##### `close(): void`

Clean up FFI resources. Always call when done.

```typescript
bridge.close();
```

### Handler Types

#### `HandlerDef<TInput, TOutput>`

Tool handler definition.

```typescript
interface HandlerDef<TInput = unknown, TOutput = unknown> {
  name: string;
  description: string;
  handler: HandlerFn<TInput, TOutput>;
  timeoutMs?: number;
}
```

#### `HandlerFn<TInput, TOutput>`

Handler function signature. Can be sync or async.

```typescript
type HandlerFn<TInput, TOutput> = (
  input: TInput,
  context: HandlerContext,
) => Promise<HandlerResult<TOutput>> | HandlerResult<TOutput>;
```

#### `HandlerResult<T>`

Handler execution result.

```typescript
type HandlerResult<T> =
  | { success: true; data: T }
  | { success: false; error: string };
```

#### `HandlerContext`

Execution context passed to handlers.

```typescript
interface HandlerContext {
  handlerName: string;
  timestamp: Date;
  log: Logger;
}
```

## 🔧 Examples

### Basic Tool

```typescript
bridge.register({
  name: "uppercase",
  description: "Convert text to uppercase",
  handler: (input: { text: string }) => ({
    success: true,
    data: { result: input.text.toUpperCase() },
  }),
});
```

### Async Tool with Validation

```typescript
bridge.register({
  name: "fetch_user",
  description: "Fetch user data",
  handler: async (input: { userId: number }) => {
    // Validate input
    if (input.userId < 1) {
      return {
        success: false,
        error: "Invalid user ID",
      };
    }

    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 100));

    return {
      success: true,
      data: {
        id: input.userId,
        name: "Alice",
        email: "alice@example.com",
      },
    };
  },
  timeoutMs: 5000,
});
```

### Error Handling

```typescript
bridge.register({
  name: "divide",
  description: "Divide two numbers",
  handler: (input: { a: number; b: number }) => {
    if (input.b === 0) {
      return {
        success: false,
        error: "Cannot divide by zero",
      };
    }

    return {
      success: true,
      data: { result: input.a / input.b },
    };
  },
});

// Execute with error
const result = await bridge.execute("divide", { a: 10, b: 0 });
if (!result.success) {
  console.error(result.error); // => "Cannot divide by zero"
}
```

### Using Context

```typescript
bridge.register({
  name: "log_operation",
  description: "Operation with logging",
  handler: (input: { value: number }, context) => {
    context.log.info(`Processing value: ${input.value}`);
    context.log.debug(`Handler: ${context.handlerName}`);
    context.log.debug(`Time: ${context.timestamp.toISOString()}`);

    return {
      success: true,
      data: { processed: input.value * 2 },
    };
  },
});
```

### Schema Validation

Automatically validate input with runtime type checking. [Full documentation](./docs/schema-validation.md)

```typescript
import { SchemaBuilder } from "./schema.ts";

bridge.register({
  name: "create_user",
  description: "Create a new user",
  handler: (input: { name: string; age: number; email: string }) => ({
    success: true,
    data: { id: 123, ...input }
  }),
  inputSchema: SchemaBuilder.object({
    name: SchemaBuilder.string({ minLength: 1, maxLength: 100 }),
    age: SchemaBuilder.number({ min: 0, max: 120 }),
    email: SchemaBuilder.string({ minLength: 5 }),
  }, ["name", "age", "email"]), // Required fields
});

// Valid input - passes validation
const result1 = await bridge.execute("create_user", {
  name: "Alice",
  age: 30,
  email: "alice@example.com"
});
// => { success: true, data: { ... } }

// Invalid input - fails validation
const result2 = await bridge.execute("create_user", {
  name: "Alice",
  age: -5 // Invalid!
});
// => { success: false, error: "Validation failed: Field 'age' must be at least 0" }
```

**Supported types:** string (with minLength/maxLength), number (with min/max), boolean, array, object

**See also:** [Complete Schema Validation Guide](./docs/schema-validation.md)

## 🧪 Testing

### Run All Tests

```bash
deno test --unstable-ffi --allow-ffi --allow-env --allow-read tests/
```

### Test Categories

- **Unit Tests** (42 tests): Core functionality
- **Integration Tests** (22 tests): End-to-end workflows
- **Property Tests** (10 tests): Memory safety, 1000+ iterations each

### Run Benchmarks

```bash
deno bench --unstable-ffi --allow-ffi --allow-env --allow-read benchmarks/
```

## 📊 Performance

| Metric                       | Value           |
| ---------------------------- | --------------- |
| FFI Overhead (reused bridge) | ~9.5µs per call |
| Create/Close Overhead        | ~300µs          |
| Throughput (sequential)      | >100K req/s     |
| Memory per Handler           | <256 bytes      |
| Cold Start                   | <100ms          |

## 🔒 Quality Standards

- **Zero SATD**: No technical debt comments
- **Complexity ≤ 20**: Per-function cyclomatic complexity
- **100% Tests Passing**: 57/57 tests green
- **Memory Safe**: Validated with property-based tests
- **Type Safe**: Strict TypeScript compilation

## 🛠️ Development

### Project Structure

```
bridges/deno/
├── bridge.ts              # Main PforgeBridge class
├── handler.ts             # Handler registry and types
├── ffi.ts                 # Low-level FFI bindings
├── examples/
│   └── hello_server.ts    # Complete working example
├── tests/
│   ├── unit/              # Unit tests
│   ├── integration/       # Integration tests
│   └── property/          # Property-based tests
├── benchmarks/            # Performance benchmarks
└── README.md              # This file
```

### Running Examples

```bash
deno run --unstable-ffi --allow-ffi --allow-env --allow-read examples/hello_server.ts
```

### Quality Gates

```bash
# Format code
deno fmt

# Lint code
deno lint

# Run tests
deno test --unstable-ffi --allow-ffi --allow-env --allow-read tests/

# All quality gates
deno fmt --check && deno lint && deno test --unstable-ffi --allow-ffi --allow-env --allow-read tests/
```

## 🤝 Contributing

1. Follow EXTREME TDD methodology (5-minute cycles)
2. Maintain zero SATD comments
3. Keep complexity ≤ 20 per function
4. Ensure all tests pass
5. Run quality gates before committing

## 📄 License

See [LICENSE](../../LICENSE) in the pforge repository.

## 🔗 Links

- [pforge Repository](https://github.com/your-org/pforge)
- [MCP Specification](https://modelcontextprotocol.io)
- [Deno Documentation](https://deno.land/manual)

## 🙏 Acknowledgments

Built with:

- **Deno** - Secure TypeScript runtime
- **Rust pforge** - High-performance MCP runtime
- **EXTREME TDD** - Toyota Way + Test-Driven Development

---

**Status**: Production-ready for basic MCP server development

**Version**: 0.1.0

**Maintained by**: pforge team
