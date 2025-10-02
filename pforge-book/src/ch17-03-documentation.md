# Documentation

Good documentation is essential for published crates. Users discover your crate on crates.io, read the README, then dive into API docs on docs.rs. This chapter covers writing comprehensive documentation that drives adoption.

## Why Documentation Matters

Documentation serves multiple audiences:

1. **New users**: Decide if the crate solves their problem (README)
2. **Integrators**: Learn how to use the API (docs.rs)
3. **Contributors**: Understand implementation (inline comments)
4. **Future you**: Remember why you made certain decisions

**Impact on adoption**: Well-documented crates get 10x more downloads than poorly documented ones with identical functionality.

## Documentation Layers

pforge uses a three-layer documentation strategy:

### Layer 1: README (Discovery)

Purpose: Convince users to try your crate

Location: `README.md` in crate root

Length: 100-200 lines

Content:
- One-line description
- Installation instructions
- Quick example (working code in 10 lines)
- Feature highlights
- Links to full documentation

### Layer 2: API Documentation (Integration)

Purpose: Teach users how to use the API

Location: Doc comments in source code

Generated: docs.rs automatic build

Content:
- Crate-level overview (`lib.rs`)
- Module documentation
- Function/struct/trait documentation
- Examples for every public API
- Usage patterns

### Layer 3: Specification (Architecture)

Purpose: Explain design decisions and architecture

Location: `docs/` directory or separate documentation site

Length: As long as needed (pforge spec is 2400+ lines)

Content:
- System architecture
- Design rationale
- Performance characteristics
- Advanced usage patterns
- Migration guides

## Writing Effective Doc Comments

Rust doc comments use `///` for items and `//!` for modules/crates.

### Crate-Level Documentation

In `lib.rs`:

```rust
//! # pforge-config
//!
//! Configuration parsing and validation for pforge MCP servers.
//!
//! This crate provides the core types and functions for parsing YAML
//! configurations into type-safe Rust structures. It validates
//! configurations against the MCP server schema.
//!
//! ## Quick Example
//!
//! ```rust
//! use pforge_config::ForgeConfig;
//!
//! let yaml = r#"
//! forge:
//!   name: my-server
//!   version: 0.1.0
//! tools:
//!   - name: greet
//!     type: native
//!     description: "Greet the user"
//! "#;
//!
//! let config = ForgeConfig::from_yaml(yaml)?;
//! assert_eq!(config.name, "my-server");
//! assert_eq!(config.tools.len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Features
//!
//! - **Type-safe parsing**: YAML → Rust structs with validation
//! - **Schema validation**: Ensures all required fields present
//! - **Error reporting**: Detailed error messages with line numbers
//! - **Zero-copy**: References into YAML string where possible
//!
//! ## Architecture
//!
//! The configuration system uses three main types:
//!
//! - [`ForgeConfig`]: Root configuration structure
//! - [`ToolDef`]: Tool definition enum (Native, CLI, HTTP, Pipeline)
//! - [`ParamSchema`]: Parameter type definitions with validation
//!
//! See the `types` module for details.

pub mod types;
pub mod validation;
pub mod parser;
```

**Key elements**:
- Title (`# pforge-config`)
- One-line description
- Quick example with complete, runnable code
- Feature highlights
- Architecture overview
- Links to modules

### Module Documentation

```rust
//! Tool definition types and validation.
//!
//! This module contains the core types for defining MCP tools:
//!
//! - [`ToolDef`]: Enum of tool types (Native, CLI, HTTP, Pipeline)
//! - [`NativeToolDef`]: Rust handler configuration
//! - [`CliToolDef`]: CLI wrapper configuration
//!
//! ## Example
//!
//! ```rust
//! use pforge_config::types::{ToolDef, NativeToolDef};
//!
//! let tool = ToolDef::Native(NativeToolDef {
//!     name: "greet".to_string(),
//!     description: "Greet the user".to_string(),
//!     handler: "greet::handler".to_string(),
//!     params: vec![],
//! });
//! ```

pub enum ToolDef {
    Native(NativeToolDef),
    Cli(CliToolDef),
    Http(HttpToolDef),
    Pipeline(PipelineToolDef),
}
```

### Function Documentation

```rust
/// Parses a YAML string into a [`ForgeConfig`].
///
/// This function validates the YAML structure and all required fields.
/// It returns detailed error messages if validation fails.
///
/// # Arguments
///
/// * `yaml` - YAML configuration string
///
/// # Returns
///
/// - `Ok(ForgeConfig)` if parsing and validation succeed
/// - `Err(ConfigError)` with detailed error message if validation fails
///
/// # Errors
///
/// Returns [`ConfigError::ParseError`] if YAML is malformed.
/// Returns [`ConfigError::ValidationError`] if required fields are missing.
///
/// # Examples
///
/// ```rust
/// use pforge_config::ForgeConfig;
///
/// let yaml = r#"
/// forge:
///   name: test-server
///   version: 0.1.0
/// "#;
///
/// let config = ForgeConfig::from_yaml(yaml)?;
/// assert_eq!(config.name, "test-server");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Invalid YAML
///
/// ```rust
/// use pforge_config::ForgeConfig;
///
/// let yaml = "invalid: yaml: content:";
/// let result = ForgeConfig::from_yaml(yaml);
/// assert!(result.is_err());
/// ```
pub fn from_yaml(yaml: &str) -> Result<ForgeConfig, ConfigError> {
    // Implementation
}
```

**Documentation sections**:
- Summary line
- Detailed description
- Arguments (with types)
- Returns (success and error cases)
- Errors (when and why they occur)
- Examples (both success and failure cases)

### Struct Documentation

```rust
/// Configuration for a Native Rust handler.
///
/// Native handlers are compiled into the server binary for maximum
/// performance. They execute with <1μs dispatch overhead.
///
/// # Fields
///
/// - `name`: Tool name (must be unique per server)
/// - `description`: Human-readable description (shown in MCP clients)
/// - `handler`: Rust function path (e.g., "handlers::greet::execute")
/// - `params`: Parameter definitions with types and validation
/// - `timeout_ms`: Optional execution timeout in milliseconds
///
/// # Example
///
/// ```rust
/// use pforge_config::types::NativeToolDef;
///
/// let tool = NativeToolDef {
///     name: "calculate".to_string(),
///     description: "Perform calculation".to_string(),
///     handler: "calc::handler".to_string(),
///     params: vec![],
///     timeout_ms: Some(5000),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeToolDef {
    pub name: String,
    pub description: String,
    pub handler: String,
    pub params: Vec<ParamSchema>,
    pub timeout_ms: Option<u64>,
}
```

### Trait Documentation

```rust
/// Handler trait for MCP tools.
///
/// Implement this trait for each tool in your server. The runtime
/// automatically registers handlers and routes requests.
///
/// # Type Parameters
///
/// - `Input`: Request parameter type (must implement `Deserialize`)
/// - `Output`: Response type (must implement `Serialize`)
///
/// # Example
///
/// ```rust
/// use pforge_runtime::Handler;
/// use async_trait::async_trait;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct GreetInput {
///     name: String,
/// }
///
/// #[derive(Serialize)]
/// struct GreetOutput {
///     message: String,
/// }
///
/// struct GreetHandler;
///
/// #[async_trait]
/// impl Handler for GreetHandler {
///     type Input = GreetInput;
///     type Output = GreetOutput;
///
///     async fn execute(&self, input: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>> {
///         Ok(GreetOutput {
///             message: format!("Hello, {}!", input.name),
///         })
///     }
/// }
/// ```
///
/// # Performance
///
/// Handler dispatch has <1μs overhead. Most time is spent in your
/// implementation. Use `async` for I/O-bound operations, avoid blocking.
///
/// # Error Handling
///
/// Return `Err` for failures. Errors are automatically converted to
/// MCP error responses with appropriate error codes.
#[async_trait]
pub trait Handler: Send + Sync {
    type Input: DeserializeOwned;
    type Output: Serialize;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Box<dyn std::error::Error>>;
}
```

## Documentation Best Practices

### 1. Write Examples That Compile

Use doc tests that actually run:

```rust
/// ```rust
/// use pforge_config::ForgeConfig;
///
/// let config = ForgeConfig::from_yaml("...")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

The `# Ok::<(), Box<dyn std::error::Error>>(())` line is hidden in rendered docs but makes the example compile.

**Test your examples**:

```bash
cargo test --doc
```

This runs all code examples. Failing examples = bad documentation.

### 2. Show Both Success and Failure

Document error cases:

```rust
/// # Examples
///
/// ## Success
///
/// ```rust
/// let result = parse("valid input");
/// assert!(result.is_ok());
/// ```
///
/// ## Invalid Input
///
/// ```rust
/// let result = parse("invalid");
/// assert!(result.is_err());
/// ```
```

Users need to know what can go wrong.

### 3. Use Intra-Doc Links

Link to related items:

```rust
/// See also [`ToolDef`] and [`ForgeConfig`].
///
/// Uses the `Handler` trait trait.
```

Makes navigation easy on docs.rs.

### 4. Document Panics

If a function can panic, document when:

```rust
/// # Panics
///
/// Panics if the handler registry is not initialized.
/// Call `Registry::init()` before using this function.
```

Though **pforge policy: no panics in production code**.

### 5. Document Safety

For `unsafe` code:

```rust
/// # Safety
///
/// Caller must ensure `ptr` is:
/// - Non-null
/// - Properly aligned
/// - Valid for reads of `len` bytes
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> &[u8] {
    // ...
}
```

### 6. Provide Context

Explain **why**, not just **what**:

**Bad**:

```rust
/// Returns the handler registry.
pub fn registry() -> &Registry { ... }
```

**Good**:

```rust
/// Returns the global handler registry.
///
/// The registry contains all registered tools and routes requests
/// to appropriate handlers. This is initialized once at startup
/// and shared across all requests for zero-overhead dispatch.
pub fn registry() -> &Registry { ... }
```

### 7. Document Performance

For performance-critical APIs:

```rust
/// Dispatches a tool call to the appropriate handler.
///
/// # Performance
///
/// - Lookup: O(1) average case using FxHash
/// - Dispatch: <1μs overhead
/// - Memory: Zero allocations for most calls
///
/// Benchmark results (Intel i7-9700K):
/// - Sequential: 1.2M calls/sec
/// - Concurrent (8 threads): 6.5M calls/sec
```

Users care about performance characteristics.

## docs.rs Configuration

docs.rs automatically builds documentation for published crates.

### Default Configuration

docs.rs builds with:
- Latest stable Rust
- Default features
- `--all-features` flag

### Custom Build Configuration

For advanced control, add `[package.metadata.docs.rs]` to `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

This enables all features for documentation builds.

### Feature Flags in Docs

Show which items require features:

```rust
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub struct HttpToolDef {
    // ...
}
```

On docs.rs, this shows "Available on crate feature `http` only".

### Platform-Specific Docs

For platform-specific items:

```rust
#[cfg(target_os = "linux")]
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
pub fn linux_specific() {
    // ...
}
```

Shows "Available on Linux only" in docs.

## Testing Documentation

### Doc Tests

Every `///` example is a test:

```rust
/// ```rust
/// use pforge_config::ForgeConfig;
/// let config = ForgeConfig::from_yaml("...")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

Run with:

```bash
cargo test --doc
```

### No-Run Examples

For examples that shouldn't execute:

```rust
/// ```rust,no_run
/// // This would connect to a real server
/// let server = Server::connect("http://example.com")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

### Compile-Only Examples

For examples that compile but shouldn't run:

```rust
/// ```rust,compile_fail
/// // This should NOT compile
/// let x: u32 = "string";
/// ```
```

Useful for demonstrating what **doesn't** work.

### Ignored Examples

For pseudo-code:

```rust
/// ```rust,ignore
/// // Simplified pseudocode
/// for tool in tools {
///     process(tool);
/// }
/// ```
```

## README Template

Here's pforge's README template:

```markdown
# pforge-config

[![Crates.io](https://img.shields.io/crates/v/pforge-config.svg)](https://crates.io/crates/pforge-config)
[![Documentation](https://docs.rs/pforge-config/badge.svg)](https://docs.rs/pforge-config)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Configuration parsing and validation for pforge MCP servers.

## Overview

pforge-config provides type-safe YAML configuration parsing for the pforge
framework. It validates configurations against the MCP server schema and
provides detailed error messages.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pforge-config = "0.1.0"
```

## Quick Example

```rust
use pforge_config::ForgeConfig;

let yaml = r#"
forge:
  name: my-server
  version: 0.1.0
tools:
  - name: greet
    type: native
    description: "Greet the user"
    handler: "handlers::greet"
"#;

let config = ForgeConfig::from_yaml(yaml)?;
println!("Server: {}", config.name);
println!("Tools: {}", config.tools.len());
```

## Features

- **Type-safe parsing**: YAML → validated Rust structs
- **Schema validation**: Ensures all required fields present
- **Detailed errors**: Line numbers and field context
- **Zero-copy**: Efficient parsing with minimal allocations
- **Extensible**: Easy to add custom validation rules

## Documentation

Full API documentation: https://docs.rs/pforge-config

For the complete pforge framework: https://github.com/paiml/pforge

## Examples

See `examples/` directory:
- `basic_config.rs`: Simple configuration
- `validation.rs`: Error handling
- `advanced.rs`: Complex configurations

Run an example:

```bash
cargo run --example basic_config
```

## Performance

- Parse time: <10ms for typical configs
- Memory usage: ~1KB per tool definition
- Validation: <1ms after parsing

## Contributing

Contributions welcome! See CONTRIBUTING.md.

## License

MIT License. See LICENSE file for details.

## Related Crates

- `pforge-runtime`: Core runtime
- `pforge-codegen`: Code generation
- `pforge-cli`: Command-line tool
```

## Documentation Checklist

Before publishing, verify:

### Crate-Level Documentation
- [ ] `lib.rs` has comprehensive `//!` documentation
- [ ] Quick example is present and compiles
- [ ] Feature list is complete
- [ ] Architecture overview explains key types
- [ ] Links to important modules work

### API Documentation
- [ ] All public functions documented
- [ ] All public structs/enums documented
- [ ] All public traits documented
- [ ] Examples for complex APIs
- [ ] Error cases documented
- [ ] Performance characteristics noted where relevant

### Examples
- [ ] Examples compile: `cargo test --doc`
- [ ] Examples are realistic (not toy examples)
- [ ] Both success and error cases shown
- [ ] Examples use proper error handling

### README
- [ ] One-line description matches `Cargo.toml`
- [ ] Installation instructions correct
- [ ] Quick example works
- [ ] Links to docs.rs and repository
- [ ] Badges are present and correct

### Building
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] No warnings: `cargo doc --no-deps 2>&1 | grep warning`
- [ ] Links resolve correctly
- [ ] Code examples all pass

## Common Documentation Mistakes

### 1. Missing Examples

**Problem**: Documentation without examples.

**Fix**: Every public API should have at least one example.

### 2. Outdated Examples

**Problem**: Examples that don't compile.

**Fix**: Run `cargo test --doc` regularly. Add to CI.

### 3. Vague Descriptions

**Problem**: "Gets the value" (what value? when? why?)

**Fix**: Be specific. "Gets the configuration value for the given key, returning None if the key doesn't exist."

### 4. Missing Error Documentation

**Problem**: Function returns `Result` but doesn't document errors.

**Fix**: Add `# Errors` section listing when each error occurs.

### 5. Broken Links

**Problem**: Links to non-existent items.

**Fix**: Use intra-doc links: `[`FunctionName`]` instead of manual URLs.

## Documentation Automation

Create a script to verify documentation:

```bash
#!/bin/bash
# scripts/check-docs.sh

set -e

echo "Checking documentation..."

# Build docs
echo "Building documentation..."
cargo doc --no-deps --all

# Test doc examples
echo "Testing doc examples..."
cargo test --doc --all

# Check for warnings
echo "Checking for warnings..."
cargo doc --no-deps --all 2>&1 | tee /tmp/doc-output.txt
if grep -q "warning" /tmp/doc-output.txt; then
    echo "ERROR: Documentation has warnings"
    exit 1
fi

# Check README examples compile
echo "Checking README examples..."
# Extract code blocks from README and test them
# (implementation depends on your needs)

echo "Documentation checks passed!"
```

Add to CI:

```yaml
# .github/workflows/ci.yml
- name: Check documentation
  run: ./scripts/check-docs.sh
```

## Summary

Comprehensive documentation requires:

1. **Three layers**: README (discovery), API docs (integration), specs (architecture)
2. **Doc comments**: Crate, module, function, struct, trait levels
3. **Examples**: Compilable, realistic, covering success and error cases
4. **Best practices**: Intra-doc links, error documentation, performance notes
5. **Testing**: `cargo test --doc` to verify examples
6. **Automation**: Scripts and CI to catch regressions

pforge's documentation strategy:
- Comprehensive `lib.rs` documentation with examples
- Every public API has examples
- README focuses on quick start
- Full specification in separate docs
- All examples tested in CI

Good documentation drives adoption and reduces support burden.

---

**Next**: [Publishing Process](ch17-04-publishing.md)
