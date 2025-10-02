# YAML Configuration: Declaring Your Calculator

The calculator's YAML configuration is **26 lines** that replace hundreds of lines of SDK boilerplate. Let's build it following EXTREME TDD principles.

## The Complete Configuration

Here's the full `forge.yaml` for our calculator server:

```yaml
forge:
  name: calculator-server
  version: 0.1.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: calculate
    description: "Perform arithmetic operations (add, subtract, multiply, divide)"
    handler:
      path: handlers::calculate_handler
    params:
      operation:
        type: string
        required: true
        description: "The operation to perform: add, subtract, multiply, or divide"
      a:
        type: float
        required: true
        description: "First operand"
      b:
        type: float
        required: true
        description: "Second operand"
```

## Section-by-Section Breakdown

### 1. Forge Metadata

```yaml
forge:
  name: calculator-server
  version: 0.1.0
  transport: stdio
  optimization: release
```

**Key decisions**:
- `name`: Unique identifier for your server
- `version`: Semantic versioning (important for client compatibility)
- `transport: stdio`: Standard input/output (most common for MCP)
- `optimization: release`: Build with optimizations enabled (<1μs dispatch)

**Alternative transports**:
- `sse`: Server-Sent Events (web-based)
- `websocket`: WebSocket (bidirectional streaming)

For local tools like calculators, `stdio` is the right choice.

### 2. Tool Definition

```yaml
tools:
  - type: native
    name: calculate
    description: "Perform arithmetic operations (add, subtract, multiply, divide)"
```

**Why a single tool?**

Instead of four separate tools (`add`, `subtract`, `multiply`, `divide`), we use **one tool with an operation parameter**. Benefits:

1. **Cleaner API**: Clients see one tool, not four
2. **Shared logic**: Validation happens once
3. **Easier testing**: Test one handler, not four
4. **Better UX**: "I want to calculate" vs "I want to add or subtract or..."

**The description field** is critical - it's what LLMs see when deciding which tool to use. Make it specific and actionable.

### 3. Handler Path

```yaml
    handler:
      path: handlers::calculate_handler
```

This tells pforge where to find your Rust handler:
- **Module**: `handlers` (the `src/handlers.rs` file)
- **Symbol**: `calculate_handler` (the exported handler struct)

**Convention**: Use `{module}::{handler_name}` format. The handler must implement the `Handler` trait.

### 4. Parameter Schema

```yaml
    params:
      operation:
        type: string
        required: true
        description: "The operation to perform: add, subtract, multiply, or divide"
      a:
        type: float
        required: true
        description: "First operand"
      b:
        type: float
        required: true
        description: "Second operand"
```

**Parameter types**:
- `string`: For operation names ("add", "subtract", etc.)
- `float`: For `f64` numeric values (supports decimals)
- `required: true`: Validation fails if missing

**Why `float` not `number`?**

MCP/JSON Schema distinguishes:
- `integer`: Whole numbers only
- `float`: Decimal/floating-point numbers

Our calculator supports `10.5 + 3.7`, so we need `float`.

## Type Safety in Action

pforge uses this YAML to generate Rust types. The params:

```yaml
params:
  operation: { type: string, required: true }
  a: { type: float, required: true }
  b: { type: float, required: true }
```

Become this Rust struct (auto-generated):

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateInput {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}
```

**No runtime validation needed** - the type system guarantees correctness!

## EXTREME TDD: Configuration First

In our 5-minute cycles, the YAML came **before** the handler:

**Cycle 0 (3 minutes)**:
1. **RED**: Create empty `forge.yaml`, run `pforge build` → fails (no handler)
2. **GREEN**: Add forge metadata and basic tool structure
3. **REFACTOR**: Add parameter descriptions

This **design-first approach** forces you to think about:
- What inputs do I need?
- What types make sense?
- What's the API contract?

## Common YAML Patterns

### Pattern 1: Optional Parameters

```yaml
params:
  operation: { type: string, required: true }
  precision: { type: integer, required: false, default: 2 }
```

### Pattern 2: Enum Constraints

```yaml
params:
  operation:
    type: string
    required: true
    enum: ["add", "subtract", "multiply", "divide"]
```

**We didn't use enum constraints** because we validate in Rust, giving better error messages.

### Pattern 3: Nested Objects

```yaml
params:
  calculation:
    type: object
    required: true
    properties:
      operation: { type: string }
      operands:
        type: array
        items: { type: float }
```

### Pattern 4: Arrays

```yaml
params:
  numbers:
    type: array
    required: true
    items: { type: float }
    minItems: 2
```

## Validation Strategy

**Two-layer validation**:

1. **YAML validation** (at build time):
   - pforge validates against its schema
   - Catches: missing required fields, invalid types
   - Fast fail: Won't even compile

2. **Runtime validation** (in handler):
   - Check operation is valid
   - Check division by zero
   - Custom business logic

**Philosophy**: Use the type system first, runtime validation second.

## Configuration vs. Code

Traditional MCP SDK (TypeScript):

```typescript
// 50+ lines of boilerplate
const server = new Server({
  name: "calculator-server",
  version: "0.1.0"
}, {
  capabilities: {
    tools: {}
  }
});

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [{
    name: "calculate",
    description: "Perform arithmetic operations",
    inputSchema: {
      type: "object",
      properties: {
        operation: { type: "string", description: "..." },
        a: { type: "number", description: "..." },
        b: { type: "number", description: "..." }
      },
      required: ["operation", "a", "b"]
    }
  }]
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === "calculate") {
    // ... handler logic
  }
});
```

pforge equivalent:

```yaml
# 26 lines, zero boilerplate
forge:
  name: calculator-server
  version: 0.1.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: calculate
    # ... (see above)
```

**90% less code. 100% type-safe. 16x faster.**

## Build-Time Code Generation

When you run `pforge build`, this YAML generates:

1. **Handler registry**: O(1) lookup for "calculate" tool
2. **Type definitions**: `CalculateInput` struct with validation
3. **JSON Schema**: For MCP protocol compatibility
4. **Dispatch logic**: Routes requests to your handler

All at **compile time** - zero runtime overhead.

## Debugging Configuration

Common errors and fixes:

**Error**: "Handler not found: handlers::calculate_handler"
```yaml
# Wrong:
handler:
  path: calculate_handler

# Right:
handler:
  path: handlers::calculate_handler
```

**Error**: "Invalid type: expected float, found string"
```yaml
# Wrong:
params:
  a: { type: string }  # User passes "5.0"

# Right:
params:
  a: { type: float }   # Parsed as 5.0
```

**Error**: "Missing required parameter: operation"
```yaml
# Wrong:
params:
  operation: { type: string }  # defaults to required: false

# Right:
params:
  operation: { type: string, required: true }
```

## Testing Your Configuration

Before writing handler code, validate your YAML:

```bash
# Validate configuration
pforge validate

# Build (validates + generates code)
pforge build --debug

# Watch mode (continuous validation)
pforge dev --watch
```

**EXTREME TDD tip**: Run `pforge validate` after every YAML edit. Fast feedback!

## Next Steps

Now that you have a valid configuration, it's time to implement the handler. Turn to Chapter 3.2 to write the Rust code that powers the calculator.

---

> "Configuration is code. Treat it with the same rigor." - pforge philosophy
