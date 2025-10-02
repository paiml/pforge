# Introduction

Welcome to **pforge** - a radical approach to building Model Context Protocol (MCP) servers that combines declarative configuration with EXTREME Test-Driven Development.

## The Problem

Building MCP servers traditionally requires:
- Hundreds of lines of boilerplate code
- Manual type safety management
- Ad-hoc quality processes
- Slow development cycles
- Runtime performance tradeoffs

## The Solution

pforge eliminates boilerplate and enforces quality through **three pillars**:

### 1. Zero-Boilerplate Configuration

Define your entire MCP server in <10 lines of YAML:

```yaml
forge:
  name: my-server
  version: 0.1.0

tools:
  - type: native
    name: greet
    description: "Greet a person"
    handler:
      path: handlers::greet
    params:
      name: { type: string, required: true }
```

### 2. EXTREME Test-Driven Development

**5-minute cycles** with strict enforcement:
1. **RED** (2 min): Write failing test
2. **GREEN** (2 min): Minimum code to pass
3. **REFACTOR** (1 min): Clean up, run quality gates
4. **COMMIT**: If gates pass
5. **RESET**: If cycle exceeds 5 minutes

**Quality gates automatically block** commits that violate:
- Code formatting (rustfmt)
- Linting (clippy -D warnings)
- Test failures
- Complexity >20
- Coverage <80%
- TDG score <75

### 3. Production Performance

pforge delivers **world-class performance** through compile-time optimization:

| Metric | Target | Achieved |
|--------|--------|----------|
| Tool dispatch | <1μs | ✅ |
| Throughput | >100K req/s | ✅ |
| Cold start | <100ms | ✅ |
| Memory/tool | <256B | ✅ |

## The EXTREME TDD Philosophy

Traditional TDD says "write tests first." EXTREME TDD says:

> **"Quality gates block bad code. Time limits prevent complexity. Automation enforces discipline."**

Key principles:
- **Jidoka (Stop the Line)**: Quality failures halt development immediately
- **Kaizen (Continuous Improvement)**: Every cycle improves the system
- **Waste Elimination**: Time-boxing prevents gold-plating
- **Amplify Learning**: Tight feedback loops accelerate mastery

## What Makes pforge Different?

### vs. Traditional MCP SDKs
- **No boilerplate**: YAML vs hundreds of lines of code
- **Compile-time safety**: Rust type system vs runtime checks
- **Performance**: <1μs dispatch vs milliseconds

### vs. Traditional TDD
- **Time-boxed**: 5-minute cycles vs indefinite
- **Automated gates**: Pre-commit hooks vs manual checks
- **Zero tolerance**: Complexity/coverage enforced vs aspirational

### vs. Quality Tools
- **Integrated**: PMAT built-in vs separate tools
- **Blocking**: Pre-commit enforcement vs reports
- **Proactive**: Prevent vs detect

## Who Should Read This Book?

This book is for you if you want to:
- Build MCP servers 10x faster
- Ship production code with confidence
- Master EXTREME TDD methodology
- Achieve <1μs performance targets
- Automate quality enforcement

### Prerequisites

- Basic Rust knowledge (or willingness to learn)
- Familiarity with Test-Driven Development
- Understanding of Model Context Protocol basics

## How to Read This Book

**Part I (Chapters 1-3)**: Learn the EXTREME TDD philosophy
- Start here if you're new to disciplined TDD
- Understand the "why" before the "how"

**Part II (Chapters 4-8)**: Build your first MCP server
- Hands-on tutorials with TDD examples
- Each chapter follows RED-GREEN-REFACTOR

**Part III (Chapters 9-12)**: Master advanced features
- State management, fault tolerance, middleware
- Real-world patterns and anti-patterns

**Part IV (Chapters 13-16)**: Quality & testing mastery
- Unit, integration, property, mutation testing
- Achieve 90%+ mutation kill rate

**Part V (Chapters 17-18)**: Performance optimization
- Sub-microsecond dispatch
- Compile-time code generation

**Part VI (Chapters 19-20)**: Production deployment
- CI/CD, multi-language bridges
- Enterprise patterns

**Part VII (Chapters 21-24)**: Real case studies
- PMAT server, data pipelines, GitHub integration
- Learn from production examples

## Code Examples

All code in this book is:
- ✅ **Tested**: 100% test coverage
- ✅ **Working**: Verified in CI/CD
- ✅ **Quality-checked**: Passed PMAT gates
- ✅ **Performant**: Benchmarked

Example code follows this format:

```rust
// Filename: src/handlers.rs
use pforge_runtime::{Handler, Result};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    message: String,
}

pub struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("Hello, {}!", input.name)
        })
    }
}
```

## Getting Help

- **Repository**: [github.com/paiml/pforge](https://github.com/paiml/pforge)
- **Issues**: [github.com/paiml/pforge/issues](https://github.com/paiml/pforge/issues)
- **Specification**: See `docs/specifications/pforge-specification.md`

## Let's Begin

The journey to EXTREME TDD starts with understanding **why** strict discipline produces better results than raw talent. Turn the page to discover the philosophy that powers pforge...

---

> "The only way to go fast is to go well." - Robert C. Martin (Uncle Bob)
