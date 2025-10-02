# Chapter 2: Quick Start

Welcome to pforge! In this chapter, you'll go from zero to a running MCP server in under 10 minutes.

## What You'll Build

By the end of this chapter, you'll have:

1. Installed pforge on your system
2. Scaffolded a new MCP server project
3. Understood the generated project structure
4. Run your first server
5. Tested it with an MCP client

## The Three-File Philosophy

A typical pforge project requires just three files:

```
my-server/
├── pforge.yaml      # Declarative configuration
├── Cargo.toml       # Rust dependencies (auto-generated)
└── src/
    └── handlers.rs  # Your business logic
```

That's it. No boilerplate, no ceremony, just your configuration and handlers.

## Why So Fast?

Traditional MCP server development requires:
- Setting up project structure
- Implementing protocol handlers
- Writing serialization/deserialization code
- Configuring transport layers
- Managing schema generation

pforge generates all of this from your YAML configuration:

```yaml
forge:
  name: my-server
  version: 0.1.0

tools:
  - type: native
    name: greet
    description: "Say hello"
    handler:
      path: handlers::greet_handler
    params:
      name: { type: string, required: true }
```

This 10-line YAML declaration produces a fully functional MCP server with:
- Type-safe input validation
- JSON Schema generation
- Error handling
- Transport configuration
- Tool registration
- Handler dispatch

## Performance Out of the Box

Your first server will achieve production-grade performance:

- Tool dispatch: <1 microsecond
- Cold start: <100 milliseconds
- Memory overhead: <512KB
- Throughput: >100K requests/second

These aren't aspirational goals - they're guaranteed by pforge's compile-time code generation.

## The EXTREME TDD Journey

As you build your server, you'll follow EXTREME TDD methodology:

1. Write a failing test (RED phase)
2. Implement minimal code to pass (GREEN phase)
3. Refactor and run quality gates (REFACTOR phase)

Each cycle takes 5 minutes or less. Quality gates automatically enforce:
- Code formatting (rustfmt)
- Linting (clippy)
- Test coverage (>80%)
- Complexity limits (<20)
- Technical debt grade (>75)

## What This Chapter Covers

### [Installation](ch02-01-installation.md)
Learn how to install pforge from crates.io or build from source. Verify your installation with diagnostic commands.

### [Your First Server](ch02-02-first-server.md)
Scaffold a new project and understand the generated structure. Explore the YAML configuration and handler implementation.

### [Testing Your Server](ch02-03-testing.md)
Run your server and test it with an MCP client. Learn basic debugging and troubleshooting techniques.

## Prerequisites

You'll need:

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- Basic terminal/command line familiarity
- A text editor (VS Code, Vim, etc.)

That's all. No complex environment setup, no Docker, no additional services.

## Time Investment

- Installation: 2 minutes
- First server: 5 minutes
- Testing: 3 minutes
- **Total: 10 minutes**

## What You Won't Learn (Yet)

This chapter focuses on getting you productive quickly. We'll cover advanced topics later:

- Multiple handler types (CLI, HTTP, Pipeline) - Chapter 5
- State management - Chapter 9
- Error handling patterns - Chapter 10
- Performance optimization - Chapter 17
- Production deployment - Chapter 19

For now, let's get your development environment set up and build your first server.

## Support

If you get stuck:

1. Check the [GitHub Issues](https://github.com/paiml/pforge/issues)
2. Review the [full specification](../../docs/specifications/pforge-specification.md)
3. Examine the [examples](../examples/) directory

Ready? Let's begin with installation.

---

Next: [Installation](ch02-01-installation.md)
