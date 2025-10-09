# pforge Deno/TypeScript Bridge v0.1.0

We're excited to announce the initial release of the **pforge Deno/TypeScript Bridge** - a high-performance, type-safe bridge for building Model Context Protocol (MCP) servers using TypeScript and Deno!

## 🎉 What's New

This is the initial release bringing full MCP server development capabilities to the Deno ecosystem with native Rust performance via FFI.

### ✨ Key Features

#### Type-Safe MCP Server Development
```typescript
import { PforgeBridge } from "https://raw.githubusercontent.com/paiml/pforge/main/bridges/deno/bridge.ts";

const bridge = new PforgeBridge();

bridge.register({
  name: "greet",
  description: "Greet a user by name",
  handler: (input: { name: string }) => ({
    success: true,
    data: { message: `Hello, ${input.name}!` },
  }),
});

const result = await bridge.execute("greet", { name: "Alice" });
// => { success: true, data: { message: "Hello, Alice!" } }
```

#### Runtime Schema Validation (No External Dependencies!)
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
  }, ["name", "age", "email"]),
});

// Automatically validates input before calling handler
const result = await bridge.execute("create_user", {
  name: "Alice",
  age: -5  // Validation fails!
});
// => { success: false, error: "Validation failed: Field 'age' must be at least 0" }
```

### 🚀 Performance

| Metric | Value |
|--------|-------|
| FFI Overhead (reused bridge) | ~9.5µs per call |
| Create/Close Overhead | ~300µs |
| Throughput (sequential) | >100K req/s |
| Memory per Handler | <256 bytes |
| Cold Start | <100ms |

### 📊 Quality Metrics

- **74 Tests Passing** (100% success rate)
  - 42 unit tests
  - 22 integration tests
  - 10 property-based tests (1000+ iterations each)
- **Zero SATD** (Self-Admitted Technical Debt) comments
- **Complexity ≤ 20** per function
- **Strict TypeScript** compilation
- **Memory Safe** - validated with property-based testing

### 📦 What's Included

#### Core API
- **`PforgeBridge`** - Main class for building MCP servers
  - `register()` - Register tool handlers
  - `execute()` - Execute tools by name
  - `list()` - List registered tools
  - `has()` - Check if tool exists
  - `count()` - Get handler count
  - `version()` - Get pforge version

#### Schema Validation
- **`SchemaBuilder`** - Ergonomic schema definitions
  - `string()` - String validation with minLength/maxLength
  - `number()` - Number validation with min/max
  - `boolean()` - Boolean validation
  - `array()` - Array validation
  - `object()` - Object validation with required fields

#### Developer Experience
- **Type-safe handlers** with full TypeScript type inference
- **Clear error messages** for validation failures
- **Both sync and async** handler support
- **Configurable timeouts** per handler
- **Comprehensive documentation** (1000+ lines)

### 📚 Documentation

- **[README.md](bridges/deno/README.md)** - Getting started guide with complete API reference
- **[docs/schema-validation.md](bridges/deno/docs/schema-validation.md)** - Comprehensive schema validation guide
- **[CHANGELOG.md](bridges/deno/CHANGELOG.md)** - Detailed changelog
- **[examples/hello_server.ts](bridges/deno/examples/hello_server.ts)** - Working example server

### 🛠️ Installation

#### Prerequisites
1. **Deno** (v2.0+): [Install Deno](https://deno.land/#installation)
2. **Rust pforge library**: Build the FFI bridge
   ```bash
   cd pforge  # Navigate to pforge root
   cargo build -p pforge-bridge --release
   ```

#### Usage
```typescript
import { PforgeBridge } from "https://raw.githubusercontent.com/paiml/pforge/main/bridges/deno/bridge.ts";
```

### 🧪 Run Tests

```bash
deno test --unstable-ffi --allow-ffi --allow-env --allow-read tests/
```

### 🏃 Run Examples

```bash
deno run --unstable-ffi --allow-ffi --allow-env --allow-read examples/hello_server.ts
```

### 📈 Run Benchmarks

```bash
deno bench --unstable-ffi --allow-ffi --allow-env --allow-read benchmarks/
```

## 🔧 Technical Implementation

Developed using **EXTREME TDD** methodology with strict 5-minute RED-GREEN-REFACTOR-COMMIT cycles:

- **Cycles 1-5**: FFI Interface (library loading, wrapper, error handling, memory safety, benchmarks)
- **Cycles 6-8**: Handler System (interface design, FFI integration, example server)
- **Cycle 9**: README documentation (465 lines)
- **Cycle 10**: JSON Schema Validation (complete implementation with 17 tests)

### Architecture

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

## 🎯 Use Cases

Perfect for:
- **AI Tool Development** - Build MCP tools for LLM integrations
- **API Wrappers** - Create type-safe MCP interfaces for REST APIs
- **Data Processing** - Build data transformation tools
- **System Integration** - Connect different services via MCP
- **Prototyping** - Rapid MCP server development with TypeScript

## 🔮 Future Plans

Planned for future releases:
- Array item type validation
- Nested object schema validation
- Custom validation functions
- Regex pattern matching for strings
- Enum/union type support
- Middleware system
- State management
- Advanced examples (HTTP client, database integration)

## 🙏 Acknowledgments

Built with:
- **Deno** - Secure TypeScript runtime
- **Rust pforge** - High-performance MCP runtime
- **EXTREME TDD** - Toyota Way + Test-Driven Development methodology

## 📝 Contributing

We welcome contributions! Please see:
- [CLAUDE.md](../../CLAUDE.md) - Development workflow and standards
- [Contributing Guidelines](../../CONTRIBUTING.md) - How to contribute

## 📄 License

MIT - see [LICENSE](../../LICENSE)

## 🔗 Links

- **Documentation**: [bridges/deno/README.md](bridges/deno/README.md)
- **Schema Guide**: [bridges/deno/docs/schema-validation.md](bridges/deno/docs/schema-validation.md)
- **Changelog**: [bridges/deno/CHANGELOG.md](bridges/deno/CHANGELOG.md)
- **Main Repository**: [https://github.com/paiml/pforge](https://github.com/paiml/pforge)
- **MCP Specification**: [https://modelcontextprotocol.io](https://modelcontextprotocol.io)
- **Deno**: [https://deno.land](https://deno.land)

## 💬 Feedback

Found a bug or have a feature request? Please [open an issue](https://github.com/paiml/pforge/issues)!

---

**Full Changelog**: [bridges/deno/CHANGELOG.md](bridges/deno/CHANGELOG.md)

**Installation**: See [bridges/deno/README.md](bridges/deno/README.md#installation)
