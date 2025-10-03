# pforge-macro

[![crates.io](https://img.shields.io/crates/v/pforge-macro.svg)](https://crates.io/crates/pforge-macro)
[![Documentation](https://docs.rs/pforge-macro/badge.svg)](https://docs.rs/pforge-macro)

Procedural macros for the pforge framework - ergonomic attribute macros for building MCP handlers.

## Features

- **`#[handler]`** - Derive Handler trait automatically
- **`#[tool]`** - Generate complete MCP tool with JSON Schema
- **Type-Safe**: Compile-time validation of handler signatures
- **Zero Boilerplate**: Write handlers as plain async functions
- **Auto Schema**: Automatic JSON Schema generation from Rust types

## Installation

```bash
cargo add pforge-macro
```

## Usage

### Basic Handler

```rust
use pforge_macro::handler;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct GreetParams {
    name: String,
}

#[handler]
async fn greet(params: GreetParams) -> Result<Value> {
    Ok(json!({
        "message": format!("Hello, {}!", params.name)
    }))
}
```

Expands to:

```rust
struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        let params: GreetParams = serde_json::from_value(params)?;
        Ok(json!({
            "message": format!("Hello, {}", params.name)
        }))
    }
}
```

### Tool Macro

The `#[tool]` macro generates everything needed for an MCP tool:

```rust
use pforge_macro::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[tool(
    name = "calculator",
    description = "Perform basic arithmetic"
)]
async fn calculate(
    #[doc = "First operand"] a: f64,
    #[doc = "Second operand"] b: f64,
    #[doc = "Operation: add, sub, mul, div"] op: String,
) -> Result<f64> {
    match op.as_str() {
        "add" => Ok(a + b),
        "sub" => Ok(a - b),
        "mul" => Ok(a * b),
        "div" if b != 0.0 => Ok(a / b),
        _ => Err(Error::Validation("Invalid operation".into())),
    }
}
```

This generates:

1. **Parameter Struct**: Type-safe params with JSON Schema
2. **Handler Implementation**: Full Handler trait
3. **Schema Function**: JSON Schema for MCP protocol
4. **Registration Helper**: Easy registry integration

### Derive Macros

```rust
use pforge_macro::Handler;
use serde::{Deserialize, Serialize};

#[derive(Handler)]
struct MyHandler {
    config: Config,
}

#[async_trait::async_trait]
impl MyHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        // Your logic here
        Ok(json!({"status": "ok"}))
    }
}
```

## Attributes

### `#[handler]`

Mark async functions as MCP handlers:

```rust
#[handler]
async fn my_handler(params: MyParams) -> Result<Value> {
    // Implementation
}
```

**Requirements**:
- Must be `async fn`
- First parameter must be deserializable from JSON
- Must return `Result<Value>` or `Result<impl Serialize>`

### `#[tool]`

Full MCP tool generation:

```rust
#[tool(
    name = "tool_name",           // Required
    description = "Description",  // Required
    timeout_ms = 5000,           // Optional
)]
async fn my_tool(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

**Generates**:
- Parameter struct with `JsonSchema`
- Handler implementation
- Schema function
- Registration code

### `#[derive(Handler)]`

Derive Handler trait for custom types:

```rust
#[derive(Handler)]
#[handler(
    validate = true,      // Add validation middleware
    log = true,          // Add logging middleware
    timeout_ms = 10000,  // Set timeout
)]
struct CustomHandler {
    // Fields
}
```

## Advanced Examples

### Stateful Handler

```rust
use pforge_macro::tool;
use std::sync::Arc;
use tokio::sync::RwLock;

struct Counter {
    value: Arc<RwLock<i64>>,
}

#[tool(
    name = "increment",
    description = "Increment counter"
)]
impl Counter {
    async fn increment(&self, amount: i64) -> Result<i64> {
        let mut value = self.value.write().await;
        *value += amount;
        Ok(*value)
    }
}
```

### Validation

```rust
use pforge_macro::tool;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[tool(
    name = "create_user",
    description = "Create a new user"
)]
async fn create_user(input: UserInput) -> Result<Value> {
    input.validate()?;
    // Create user
    Ok(json!({"id": "user_123"}))
}
```

### Error Handling

```rust
use pforge_macro::tool;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    Invalid(String),
}

#[tool(
    name = "get_user",
    description = "Fetch user by ID"
)]
async fn get_user(id: String) -> Result<User, MyError> {
    database::find_user(&id)
        .await
        .ok_or_else(|| MyError::NotFound(id))
}
```

## Generated Code Quality

All macro-generated code:

- ✅ Follows Rust naming conventions
- ✅ Includes proper error handling
- ✅ Generates complete documentation
- ✅ Passes clippy lints
- ✅ Is fully type-safe

## Compile-Time Checks

The macros perform validation at compile time:

- **Signature Validation**: Ensures async fn with correct return type
- **Type Bounds**: Verifies Serialize/Deserialize bounds
- **Attribute Validation**: Checks required attributes are present
- **Name Conflicts**: Prevents duplicate tool names

## Documentation

- [API Documentation](https://docs.rs/pforge-macro)
- [Macro Guide](https://github.com/paiml/pforge/blob/main/docs/USER_GUIDE.md#macros)
- [Examples](https://github.com/paiml/pforge/tree/main/examples)

## License

MIT
