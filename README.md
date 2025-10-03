# pforge

A declarative framework for building Model Context Protocol (MCP) servers using YAML configuration.

[![CI](https://github.com/paiml/pforge/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/pforge/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/pforge-cli.svg)](https://crates.io/crates/pforge-cli)
[![crates.io](https://img.shields.io/crates/v/pforge-runtime.svg)](https://crates.io/crates/pforge-runtime)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What is pforge?

pforge lets you define MCP servers in YAML instead of writing boilerplate code. It's built on top of [pmcp](https://github.com/paiml/pmcp) (rust-mcp-sdk) and generates optimized Rust code from your configuration.

**Quick example:**

```yaml
forge:
  name: my-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: greet
    description: "Greet someone"
    handler:
      path: handlers::greet_handler
    params:
      name: { type: string, required: true }
```

## Installation

```bash
# From crates.io
cargo install pforge-cli

# From source
git clone https://github.com/paiml/pforge
cd pforge
cargo install --path crates/pforge-cli
```

## Quick Start

```bash
# Create new project
pforge new my-server
cd my-server

# Run the server
pforge serve
```

The scaffolded project includes a working example handler. Edit `pforge.yaml` to add more tools, then implement handlers in `src/handlers/`.

## Handler Types

pforge supports four handler types:

1. **Native** - Rust functions with full type safety
2. **CLI** - Execute shell commands
3. **HTTP** - Proxy HTTP endpoints
4. **Pipeline** - Chain multiple tools together

See the [book](https://paiml.github.io/pforge) for detailed examples of each type.

## Documentation

- **[Book](https://paiml.github.io/pforge)** - Complete guide with examples and comparisons
- **[Architecture](docs/ARCHITECTURE.md)** - Technical design details
- **[User Guide](docs/USER_GUIDE.md)** - Usage guide
- **[Implementation Status](docs/IMPLEMENTATION_STATUS.md)** - Current project status
- **[CLAUDE.md](CLAUDE.md)** - Development workflow for contributors

## Examples

- **[hello-world](examples/hello-world/)** - Minimal native handler example
- **[calculator](examples/calculator/)** - Math operations with tests
- **[rest-api-proxy](examples/rest-api-proxy/)** - HTTP handler examples

## Project Status

**Version:** 0.1.1

**Published crates:**
- `pforge-config` - Configuration parsing
- `pforge-macro` - Procedural macros
- `pforge-runtime` - Core runtime (depends on pmcp)
- `pforge-codegen` - Code generation
- `pforge-cli` - CLI tool

**Test results:** 120+ tests passing (90+ unit/integration, 12 property-based, 8 quality gates, 5+ doctests)

See [IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) for detailed progress.

## Development

```bash
# Run tests
cargo test --all

# Run quality gates
make quality-gate

# Watch mode
make watch

# Build release
make build-release
```

See [CLAUDE.md](CLAUDE.md) for full development workflow.

## Architecture

pforge is built as a framework on top of pmcp (rust-mcp-sdk):

```
┌─────────────────────────────────┐
│   pforge (Framework Layer)      │
│   - YAML → Rust codegen         │
│   - Handler registry            │
│   - State management            │
└─────────────────────────────────┘
              ↓
┌─────────────────────────────────┐
│   pmcp (Protocol SDK)           │
│   - MCP protocol implementation │
│   - Transport handling          │
└─────────────────────────────────┘
```

When to use pmcp directly: You need fine-grained control over MCP protocol details or want to avoid code generation.

When to use pforge: You want declarative configuration and rapid MCP server development with less code.

## Contributing

Contributions are welcome. Please:

1. Read [CLAUDE.md](CLAUDE.md) for development standards
2. Check [ROADMAP.md](ROADMAP.md) for current priorities
3. Ensure tests pass: `cargo test --all`
4. Ensure quality gates pass: `make quality-gate`

All commits are validated by pre-commit hooks that check code formatting, linting, tests, complexity, coverage, and **markdown link validity** (using `pmat validate-docs`) to prevent broken documentation links.

## License

MIT - see [LICENSE](LICENSE)

## Acknowledgments

Built on [pmcp](https://github.com/paiml/pmcp) by Pragmatic AI Labs.
