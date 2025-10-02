# Installation

Installing pforge takes less than two minutes. You have two options: install from crates.io (recommended) or build from source.

## Prerequisites

Before installing pforge, ensure you have Rust installed:

```bash
# Check if Rust is installed
rustc --version

# If not installed, get it from rustup.rs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You'll need Rust 1.70 or later. pforge leverages modern Rust features for performance and safety.

## Option 1: Install from crates.io (Recommended)

The simplest installation method:

```bash
cargo install pforge-cli
```

This downloads the pre-built pforge CLI from crates.io and installs it to `~/.cargo/bin/pforge`.

Expected output:

```
    Updating crates.io index
  Downloaded pforge-cli v0.1.0
  Downloaded 1 crate (45.2 KB) in 0.89s
   Compiling pforge-cli v0.1.0
    Finished release [optimized] target(s) in 1m 23s
  Installing ~/.cargo/bin/pforge
   Installed package `pforge-cli v0.1.0` (executable `pforge`)
```

Installation typically takes 1-2 minutes depending on your connection speed and CPU.

## Option 2: Build from Source

For the latest development version or to contribute:

```bash
# Clone the repository
git clone https://github.com/paiml/pforge
cd pforge

# Build and install
cargo install --path crates/pforge-cli

# Or use the Makefile
make install
```

Building from source gives you:
- Latest features not yet published to crates.io
- Ability to modify the source code
- Development environment for contributing

Note: Source builds take longer (3-5 minutes) due to full dependency compilation.

## Verify Installation

Check that pforge is correctly installed:

```bash
pforge --version
```

Expected output:

```
pforge 0.1.0
```

Try the help command:

```bash
pforge --help
```

You should see:

```
pforge 0.1.0
A declarative framework for building MCP servers

USAGE:
    pforge <SUBCOMMAND>

SUBCOMMANDS:
    new       Create a new pforge project
    serve     Run an MCP server
    build     Build a server binary
    dev       Development mode with hot reload
    test      Run server tests
    help      Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

## Troubleshooting

### Command Not Found

If you see `command not found: pforge`, ensure `~/.cargo/bin` is in your PATH:

```bash
# Check if it's in PATH
echo $PATH | grep -q ".cargo/bin" && echo "Found" || echo "Not found"

# Add to PATH (add this to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload your shell
source ~/.bashrc  # or source ~/.zshrc
```

### Compilation Errors

If installation fails with compilation errors:

1. Update Rust to the latest stable version:

```bash
rustup update stable
rustup default stable
```

2. Clear the cargo cache and retry:

```bash
cargo clean
cargo install pforge-cli --force
```

3. Check for system dependencies (Linux):

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc pkg-config openssl-devel
```

### Network Issues

If crates.io download fails:

1. Check your internet connection
2. Try using a mirror or proxy
3. Build from source as a fallback

## Platform-Specific Notes

### macOS

pforge works out of the box on macOS 10.15 or later. For Apple Silicon (M1/M2):

```bash
# Verify architecture
uname -m  # Should show arm64

# Install normally
cargo install pforge-cli
```

### Linux

Tested on:
- Ubuntu 20.04+ (x86_64, ARM64)
- Debian 11+
- Fedora 35+
- Arch Linux (latest)

Ensure you have a C compiler (gcc or clang) installed.

### Windows

pforge supports Windows 10 and later with either:
- MSVC toolchain (recommended)
- GNU toolchain (mingw-w64)

```powershell
# Install using PowerShell
cargo install pforge-cli

# Verify
pforge --version
```

Note: Some examples use Unix-style paths. Windows users should adjust accordingly.

## Development Dependencies (Optional)

For the full development experience with quality gates:

```bash
# Install cargo-watch for hot reload
cargo install cargo-watch

# Install cargo-tarpaulin for coverage (Linux only)
cargo install cargo-tarpaulin

# Install cargo-mutants for mutation testing
cargo install cargo-mutants

# Install pmat for quality analysis
cargo install pmat
```

These are optional for basic usage but required if you plan to:
- Run quality gates (`make quality-gate`)
- Use watch mode (`pforge dev --watch`)
- Measure test coverage
- Perform mutation testing

## Updating pforge

To update to the latest version:

```bash
cargo install pforge-cli --force
```

The `--force` flag reinstalls even if the current version is up to date.

Check release notes at: https://github.com/paiml/pforge/releases

## Uninstalling

To remove pforge:

```bash
cargo uninstall pforge-cli
```

This removes the binary from `~/.cargo/bin/pforge`.

## Next Steps

Now that pforge is installed, let's create your first server.

---

Next: [Your First Server](ch02-02-first-server.md)
