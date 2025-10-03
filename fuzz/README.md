# pforge Fuzzing Infrastructure

This directory contains fuzz targets for testing pforge components with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Prerequisites

1. **Install cargo-fuzz** (if not already installed):
   ```bash
   cargo install cargo-fuzz
   ```

2. **Install build dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install -y libstdc++-11-dev build-essential

   # macOS
   xcode-select --install
   ```

3. **Use nightly Rust** (required for fuzzing):
   ```bash
   rustup install nightly
   ```

## Fuzz Targets

### 1. Config Parser Fuzzer (`fuzz_config_parser`)

Tests the YAML configuration parser for crashes and panics.

**What it tests**:
- YAML parsing robustness
- Invalid UTF-8 handling
- Malformed YAML structures
- Edge cases in config validation

**Run it**:
```bash
cargo +nightly fuzz run fuzz_config_parser
```

**Run with timeout**:
```bash
cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=300
```

### 2. Handler Dispatch Fuzzer (`fuzz_handler_dispatch`)

Tests the handler registry lookup mechanism.

**What it tests**:
- Registry lookups with arbitrary tool names
- Empty string handling
- Very long tool names (DOS protection)
- Non-UTF8 byte sequences

**Run it**:
```bash
cargo +nightly fuzz run fuzz_handler_dispatch
```

### 3. Validation Fuzzer (`fuzz_validation`)

Tests configuration validation logic.

**What it tests**:
- Tool name uniqueness
- Config roundtrip serialization
- Validation invariants
- Schema consistency

**Run it**:
```bash
cargo +nightly fuzz run fuzz_validation
```

## Running All Fuzz Targets

```bash
#!/bin/bash
# Run each fuzz target for 5 minutes
for target in fuzz_config_parser fuzz_handler_dispatch fuzz_validation; do
    echo "Fuzzing $target for 300 seconds..."
    cargo +nightly fuzz run $target -- -max_total_time=300
done
```

## CI/CD Integration

For continuous fuzzing in CI:

```bash
# Run each target for 60 seconds (quick smoke test)
cargo +nightly fuzz run fuzz_config_parser -- -max_total_time=60 -runs=10000
cargo +nightly fuzz run fuzz_handler_dispatch -- -max_total_time=60 -runs=10000
cargo +nightly fuzz run fuzz_validation -- -max_total_time=60 -runs=10000
```

## Corpus Management

Fuzz targets automatically save interesting inputs to `corpus/` directories:

```
fuzz/
├── corpus/
│   ├── fuzz_config_parser/
│   ├── fuzz_handler_dispatch/
│   └── fuzz_validation/
```

### Adding Seed Inputs

Create initial corpus entries:

```bash
# Config parser seeds
echo "forge:
  name: test
  version: 0.1.0
tools: []" > corpus/fuzz_config_parser/valid_minimal.yaml

echo "invalid yaml {{{" > corpus/fuzz_config_parser/invalid.yaml
```

## Crash Analysis

When a crash is found:

1. **Reproduce the crash**:
   ```bash
   cargo +nightly fuzz run fuzz_config_parser crash-<hash>
   ```

2. **Minimize the crash input**:
   ```bash
   cargo +nightly fuzz tmin fuzz_config_parser crash-<hash>
   ```

3. **Debug with LLDB/GDB**:
   ```bash
   cargo +nightly fuzz run --debug fuzz_config_parser crash-<hash>
   lldb ./target/x86_64-unknown-linux-gnu/release/fuzz_config_parser
   ```

## Coverage Reports

Generate coverage from fuzzing:

```bash
cargo +nightly fuzz coverage fuzz_config_parser
cargo cov -- show target/x86_64-unknown-linux-gnu/coverage/*/release/fuzz_config_parser \
    -format=html -instr-profile=coverage/fuzz_config_parser/coverage.profdata \
    > coverage.html
```

## Performance Tuning

### Increase iterations
```bash
cargo +nightly fuzz run fuzz_config_parser -- -runs=1000000
```

### Use multiple jobs
```bash
cargo +nightly fuzz run fuzz_config_parser -- -jobs=8
```

### Limit memory
```bash
cargo +nightly fuzz run fuzz_config_parser -- -rss_limit_mb=2048
```

## Troubleshooting

### "cannot find -lstdc++"

Install C++ standard library:
```bash
sudo apt-get install -y libstdc++-11-dev
```

### Fuzzing too slow

1. Use release mode (default)
2. Reduce input size: `-max_len=4096`
3. Use dictionary: `-dict=fuzz.dict`

### Out of memory

Limit RSS: `-rss_limit_mb=2048`

## References

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer options](https://llvm.org/docs/LibFuzzer.html#options)
- [AFL++ comparison](https://github.com/AFLplusplus/AFLplusplus)

## Security

If fuzzing discovers a security vulnerability:

1. Do not commit the crash input to the public repository
2. Follow the security disclosure policy in `SECURITY.md`
3. File a private security advisory on GitHub
