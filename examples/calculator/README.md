# Calculator Server Example

A simple arithmetic calculator MCP server demonstrating pforge's native handler type.

## Features

- **Four operations**: add, subtract, multiply, divide
- **Type-safe inputs**: Validated f64 parameters
- **Error handling**: Division by zero detection
- **Full test coverage**: 6 unit tests

## Configuration

See `forge.yaml` for the complete server configuration.

## Handler Implementation

The calculator logic is in `src/handlers.rs`:
- `CalculateInput`: Input parameter struct
- `CalculateOutput`: Result struct
- `CalculateHandler`: Implements the `Handler` trait

## Running Tests

```bash
cargo test
```

Expected output:
```
running 6 tests
test handlers::tests::test_add ... ok
test handlers::tests::test_subtract ... ok
test handlers::tests::test_multiply ... ok
test handlers::tests::test_divide ... ok
test handlers::tests::test_divide_by_zero ... ok
test handlers::tests::test_unknown_operation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Running the Server

```bash
# From the pforge root directory
pforge serve --config examples/calculator/forge.yaml
```

## Example Usage

```json
{
  "operation": "add",
  "a": 5.0,
  "b": 3.0
}
// Returns: {"result": 8.0}
```

## Coverage in Book

This example is covered in:
- Chapter 3: Calculator Server
- Chapter 5: Native Handlers
- Chapter 9: Unit Testing
