# Integration Testing CLI Handlers

CLI handlers bridge pforge to the system shell. This chapter covers comprehensive integration testing strategies to ensure reliability across different environments and edge cases.

## Testing Philosophy for CLI Handlers

**Unit tests** verify handler construction:
```rust
#[test]
fn test_cli_handler_creation() {
    let handler = CliHandler::new(...);
    assert_eq!(handler.command, "ls");
}
```

**Integration tests** verify actual command execution:
```rust
#[tokio::test]
async fn test_cli_handler_executes() {
    let result = handler.execute(input).await.unwrap();
    assert_eq!(result.exit_code, 0);
}
```

**This chapter focuses on integration tests.**

## Basic Integration Test Structure

```rust
use pforge_runtime::handlers::cli::{CliHandler, CliInput};
use std::collections::HashMap;

#[tokio::test]
async fn test_ls_command() {
    // Arrange
    let handler = CliHandler::new(
        "ls".to_string(),
        vec!["-lah".to_string()],
        None,  // cwd
        HashMap::new(),  // env
        Some(5000),  // timeout_ms
        false,  // stream
    );

    let input = CliInput {
        args: vec![],
        env: HashMap::new(),
    };

    // Act
    let result = handler.execute(input).await.unwrap();

    // Assert
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout.is_empty());
    assert_eq!(result.stderr, "");
}
```

## Testing Success Cases

### Command Execution Success

```rust
#[tokio::test]
async fn test_echo_command() {
    let handler = CliHandler::new(
        "echo".to_string(),
        vec!["hello world".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.trim() == "hello world");
}
```

### Argument Passing

```rust
#[tokio::test]
async fn test_grep_with_args() {
    let handler = CliHandler::new(
        "grep".to_string(),
        vec!["pattern".to_string()],
        None,
        HashMap::new(),
        Some(2000),
        false,
    );

    let input = CliInput {
        args: vec!["testfile.txt".to_string()],
        env: HashMap::new(),
    };

    let result = handler.execute(input).await.unwrap();

    // grep returns 0 if pattern found, 1 if not, >1 on error
    assert!(result.exit_code <= 1);
}
```

### Working Directory

```rust
#[tokio::test]
async fn test_pwd_in_specific_dir() {
    let test_dir = std::env::temp_dir();

    let handler = CliHandler::new(
        "pwd".to_string(),
        vec![],
        Some(test_dir.to_str().unwrap().to_string()),
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains(test_dir.to_str().unwrap()));
}
```

### Environment Variables

```rust
#[tokio::test]
async fn test_env_variables() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let handler = CliHandler::new(
        "sh".to_string(),
        vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
        None,
        env,
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("test_value"));
}
```

## Testing Failure Cases

### Command Not Found

```rust
#[tokio::test]
async fn test_nonexistent_command() {
    let handler = CliHandler::new(
        "nonexistent_command_xyz".to_string(),
        vec![],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Handler(_)));
}
```

### Non-Zero Exit Code

```rust
#[tokio::test]
async fn test_command_fails() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec!["-c".to_string(), "exit 42".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 42);
    assert!(result.stdout.is_empty());
}
```

### Timeout Exceeded

```rust
#[tokio::test]
async fn test_command_timeout() {
    let handler = CliHandler::new(
        "sleep".to_string(),
        vec!["10".to_string()],  // Sleep 10 seconds
        None,
        HashMap::new(),
        Some(100),  // Timeout after 100ms
        false,
    );

    let result = handler.execute(CliInput::default()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Timeout));
}
```

### Invalid Arguments

```rust
#[tokio::test]
async fn test_invalid_arguments() {
    let handler = CliHandler::new(
        "ls".to_string(),
        vec!["--invalid-flag-xyz".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_ne!(result.exit_code, 0);
    assert!(!result.stderr.is_empty());
}
```

## Testing Output Handling

### Stdout Capture

```rust
#[tokio::test]
async fn test_stdout_captured() {
    let handler = CliHandler::new(
        "echo".to_string(),
        vec!["line1\nline2\nline3".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert!(result.stdout.contains("line1"));
    assert!(result.stdout.contains("line2"));
    assert!(result.stdout.contains("line3"));
}
```

### Stderr Capture

```rust
#[tokio::test]
async fn test_stderr_captured() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec!["-c".to_string(), "echo error >&2".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stderr.contains("error"));
    assert_eq!(result.stdout, "");
}
```

### Large Output

```rust
#[tokio::test]
async fn test_large_output() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec![
            "-c".to_string(),
            "for i in $(seq 1 10000); do echo line$i; done".to_string(),
        ],
        None,
        HashMap::new(),
        Some(10000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    let line_count = result.stdout.lines().count();
    assert_eq!(line_count, 10000);
}
```

## Testing Streaming Handlers

### Stream Output Capture

```rust
#[tokio::test]
async fn test_streaming_output() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec![
            "-c".to_string(),
            "for i in 1 2 3; do echo line$i; sleep 0.1; done".to_string(),
        ],
        None,
        HashMap::new(),
        Some(5000),
        true,  // stream: true
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("line1"));
    assert!(result.stdout.contains("line2"));
    assert!(result.stdout.contains("line3"));
}
```

### Stream Timeout

```rust
#[tokio::test]
async fn test_stream_timeout() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec![
            "-c".to_string(),
            "echo start; sleep 10; echo end".to_string(),
        ],
        None,
        HashMap::new(),
        Some(500),  // Timeout before "end" prints
        true,
    );

    let result = handler.execute(CliInput::default()).await;

    assert!(result.is_err());
}
```

## Testing Edge Cases

### Empty Output

```rust
#[tokio::test]
async fn test_empty_output() {
    let handler = CliHandler::new(
        "true".to_string(),  // Command that succeeds but prints nothing
        vec![],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "");
    assert_eq!(result.stderr, "");
}
```

### Special Characters in Arguments

```rust
#[tokio::test]
async fn test_special_characters() {
    let handler = CliHandler::new(
        "echo".to_string(),
        vec!["$TEST".to_string(), "!@#$%".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    // Note: shell won't expand $TEST since we use Command::new, not sh -c
    assert!(result.stdout.contains("$TEST"));
}
```

### Unicode Output

```rust
#[tokio::test]
async fn test_unicode_output() {
    let handler = CliHandler::new(
        "echo".to_string(),
        vec!["Hello 世界 🚀".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("世界"));
    assert!(result.stdout.contains("🚀"));
}
```

## Platform-Specific Tests

### Unix-Only Tests

```rust
#[cfg(unix)]
#[tokio::test]
async fn test_unix_specific_command() {
    let handler = CliHandler::new(
        "uname".to_string(),
        vec!["-s".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("Linux") || result.stdout.contains("Darwin"));
}
```

### Windows-Only Tests

```rust
#[cfg(windows)]
#[tokio::test]
async fn test_windows_specific_command() {
    let handler = CliHandler::new(
        "cmd".to_string(),
        vec!["/C".to_string(), "echo test".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("test"));
}
```

## Property-Based Testing

### Random Command Arguments

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn cli_handler_never_panics(
        args in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 0..10)
    ) {
        tokio_test::block_on(async {
            let handler = CliHandler::new(
                "echo".to_string(),
                args,
                None,
                HashMap::new(),
                Some(1000),
                false,
            );

            // Should not panic, even with random args
            let _ = handler.execute(CliInput::default()).await;
        });
    }
}
```

### Exit Code Range

```rust
proptest! {
    #[test]
    fn exit_codes_are_valid(
        code in 0..=255u8
    ) {
        tokio_test::block_on(async {
            let handler = CliHandler::new(
                "sh".to_string(),
                vec!["-c".to_string(), format!("exit {}", code)],
                None,
                HashMap::new(),
                Some(1000),
                false,
            );

            let result = handler.execute(CliInput::default()).await.unwrap();
            prop_assert_eq!(result.exit_code, code as i32);
            Ok(())
        })?;
    }
}
```

## Mock Command Patterns

### Test Fixture Script

Create `tests/fixtures/test_command.sh`:

```bash
#!/bin/bash
# Test fixture for CLI handler integration tests

case "$1" in
  success)
    echo "Success output"
    exit 0
    ;;
  failure)
    echo "Error output" >&2
    exit 1
    ;;
  slow)
    sleep 5
    echo "Done"
    exit 0
    ;;
  *)
    echo "Unknown command" >&2
    exit 2
    ;;
esac
```

**Usage in tests**:

```rust
#[tokio::test]
async fn test_with_fixture() {
    let handler = CliHandler::new(
        "./tests/fixtures/test_command.sh".to_string(),
        vec!["success".to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );

    let result = handler.execute(CliInput::default()).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("Success"));
}
```

## Test Coverage Goals

### Coverage Checklist

- [x] Command execution succeeds
- [x] Command execution fails
- [x] Timeout handling
- [x] Stdout capture
- [x] Stderr capture
- [x] Exit code handling
- [x] Argument passing
- [x] Environment variables
- [x] Working directory
- [x] Streaming output
- [x] Large output
- [x] Empty output
- [x] Special characters
- [x] Unicode handling
- [x] Platform-specific behavior

### Measuring Coverage

```bash
# Run integration tests with coverage
cargo tarpaulin \
  --test integration \
  --out Html \
  --output-dir target/coverage

# View report
open target/coverage/index.html
```

**Target**: ≥80% line coverage for CLI handler code.

## Continuous Integration

### GitHub Actions Example

```yaml
name: CLI Handler Integration Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run integration tests
        run: cargo test --test cli_integration

      - name: Run with verbose output
        run: cargo test --test cli_integration -- --nocapture
```

## Best Practices

### 1. Isolate Test Dependencies

```rust
// BAD - depends on system state
#[tokio::test]
async fn test_list_home_dir() {
    let handler = CliHandler::new(
        "ls".to_string(),
        vec![std::env::var("HOME").unwrap()],  // System-dependent
        None,
        HashMap::new(),
        Some(1000),
        false,
    );
    // ...
}

// GOOD - create isolated test environment
#[tokio::test]
async fn test_list_test_dir() {
    let temp_dir = tempfile::tempdir().unwrap();

    let handler = CliHandler::new(
        "ls".to_string(),
        vec![temp_dir.path().to_str().unwrap().to_string()],
        None,
        HashMap::new(),
        Some(1000),
        false,
    );
    // ...
}
```

### 2. Test Timeouts Appropriately

```rust
// Ensure timeout is longer than expected execution
let handler = CliHandler::new(
    "sleep".to_string(),
    vec!["2".to_string()],
    None,
    HashMap::new(),
    Some(3000),  // 3s > 2s command duration
    false,
);
```

### 3. Assert on Both Success and Error Paths

```rust
#[tokio::test]
async fn test_comprehensive() {
    let result = handler.execute(input).await.unwrap();

    // Assert success conditions
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout.is_empty());

    // Assert error conditions didn't occur
    assert_eq!(result.stderr, "");
}
```

## Next Steps

Chapter 5.0 introduces HTTP handlers for wrapping REST APIs, starting with a GitHub API integration example.

---

> "Test the integration, not just the units. CLI handlers live at the system boundary." - pforge testing principle
