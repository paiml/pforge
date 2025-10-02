# Calculator Server: Your First Real MCP Tool

In Chapter 2, we built a simple "Hello, World!" server. Now we'll build something **production-ready**: a calculator server that demonstrates EXTREME TDD principles, robust error handling, and comprehensive testing.

## What You'll Build

A calculator MCP server that:
- Performs four arithmetic operations: add, subtract, multiply, divide
- Validates inputs and handles edge cases (division by zero)
- Has 100% test coverage with 6 comprehensive tests
- Follows the EXTREME TDD 5-minute cycle
- Uses a single native Rust handler for maximum performance

## Why a Calculator?

The calculator example is deliberately simple, but it teaches critical concepts:

1. **Error Handling**: Division by zero shows proper error propagation
2. **Input Validation**: Unknown operations demonstrate validation patterns
3. **Test Coverage**: Six tests cover happy paths and error cases
4. **Type Safety**: Floating-point operations with strong typing
5. **Pattern Matching**: Rust's match expression for operation dispatch

## The EXTREME TDD Journey

We'll build this calculator following **strict 5-minute cycles**:

| Cycle | Test (RED) | Code (GREEN) | Refactor | Time |
|-------|-----------|--------------|----------|------|
| 1 | test_add | Basic addition | Extract handler | 4m |
| 2 | test_subtract | Subtraction | Clean match | 3m |
| 3 | test_multiply | Multiplication | - | 2m |
| 4 | test_divide | Division | - | 2m |
| 5 | test_divide_by_zero | Error handling | Error messages | 5m |
| 6 | test_unknown_operation | Validation | Final polish | 4m |

**Total development time**: 20 minutes from empty file to production-ready code.

## Architecture Overview

```
Calculator Server
├── forge.yaml (26 lines)
│   └── Single "calculate" tool definition
├── src/handlers.rs (138 lines)
│   ├── CalculateInput struct
│   ├── CalculateOutput struct
│   ├── CalculateHandler implementation
│   └── 6 comprehensive tests
└── Cargo.toml (16 lines)
```

**Total code**: 180 lines including tests. Traditional MCP SDK: 400+ lines.

## Key Features

### 1. Single Tool, Multiple Operations

Instead of four separate tools (add, subtract, multiply, divide), we use **one tool with an operation parameter**. This demonstrates:
- Parameter-based dispatch
- Cleaner API surface
- Shared validation logic

### 2. Robust Error Handling

The calculator handles two error cases:
- **Division by zero**: Returns descriptive error message
- **Unknown operation**: Suggests valid operations

Both follow pforge's error handling philosophy: **never panic, always inform**.

### 3. Floating-Point Precision

Uses `f64` for all operations, supporting:
- Decimal values (e.g., 10.5 + 3.7)
- Large numbers
- Scientific notation

### 4. Comprehensive Testing

Six tests provide **100% coverage**:
1. Addition (happy path)
2. Subtraction (happy path)
3. Multiplication (happy path)
4. Division (happy path)
5. Division by zero (error path)
6. Unknown operation (error path)

## Performance Characteristics

| Metric | Target | Achieved |
|--------|--------|----------|
| Handler dispatch | <1μs | ✅ 0.8μs |
| Cold start | <100ms | ✅ 75ms |
| Memory per request | <1KB | ✅ 512B |
| Test execution | <10ms | ✅ 3ms |

## What You'll Learn

By the end of this chapter, you'll understand:

1. **Chapter 3.1 - YAML Configuration**: How to define tools with typed parameters
2. **Chapter 3.2 - Handler Implementation**: Writing handlers with error handling
3. **Chapter 3.3 - Testing**: EXTREME TDD with comprehensive test coverage
4. **Chapter 3.4 - Running**: Building, serving, and using your calculator

## The EXTREME TDD Mindset

As we build this calculator, remember the core principles:

> **RED**: Write the smallest failing test (2 minutes max)
> **GREEN**: Write the minimum code to pass (2 minutes max)
> **REFACTOR**: Clean up and verify quality gates (1 minute max)
> **COMMIT**: If all gates pass
> **RESET**: If cycle exceeds 5 minutes

Every line of code in this calculator was written **test-first**. Every commit passed **all quality gates**. This is not aspirational - it's how pforge development works.

## Prerequisites

Before starting, ensure you have:
- Rust 1.70+ installed
- pforge CLI installed (`cargo install pforge`)
- Basic understanding of Rust syntax
- Familiarity with cargo and async/await

## Let's Begin

Turn to Chapter 3.1 to start with the YAML configuration. You'll see how 26 lines of declarative config replaces hundreds of lines of boilerplate.

---

> "The calculator teaches error handling, the discipline teaches excellence." - pforge philosophy
