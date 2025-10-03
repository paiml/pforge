# pforge Installation Guide

**Multiple installation methods for all platforms.**

---

## Quick Install (Recommended)

### One-Line Install (Linux/macOS)
```bash
curl -sSL https://raw.githubusercontent.com/paiml/pforge/main/install.sh | bash
```

### Cargo (All Platforms)
```bash
cargo install pforge-cli
```

---

## Installation Methods

### 1. Cargo (crates.io)

**Prerequisites**: Rust 1.75+

```bash
# Install from crates.io
cargo install pforge-cli

# Verify installation
pforge --version
```

**Advantages**:
- ✅ Always latest version
- ✅ Works on all platforms
- ✅ Automatic updates with `cargo install --force`

### 2. Homebrew (macOS/Linux)

```bash
# Add pforge tap
brew tap paiml/pforge

# Install pforge
brew install pforge

# Verify installation
pforge --version
```

**Advantages**:
- ✅ System package manager
- ✅ Automatic updates with `brew upgrade`
- ✅ Shell completions included

### 3. Binary Download

**Linux (x86_64)**:
```bash
curl -LO https://github.com/paiml/pforge/releases/latest/download/pforge-linux-amd64.tar.gz
tar xzf pforge-linux-amd64.tar.gz
sudo mv pforge /usr/local/bin/
```

**macOS (Apple Silicon)**:
```bash
curl -LO https://github.com/paiml/pforge/releases/latest/download/pforge-macos-arm64.tar.gz
tar xzf pforge-macos-arm64.tar.gz
sudo mv pforge /usr/local/bin/
```

**Windows (PowerShell)**:
```powershell
Invoke-WebRequest -Uri "https://github.com/paiml/pforge/releases/latest/download/pforge-windows-amd64.zip" -OutFile pforge.zip
Expand-Archive pforge.zip
Move-Item pforge\pforge.exe C:\Windows\System32\
```

**Verify checksum**:
```bash
# Download checksum
curl -LO https://github.com/paiml/pforge/releases/latest/download/pforge-linux-amd64.tar.gz.sha256

# Verify
sha256sum -c pforge-linux-amd64.tar.gz.sha256
```

### 4. Docker

**Pull image**:
```bash
docker pull ghcr.io/paiml/pforge:latest
```

**Run server**:
```bash
docker run -it --rm \
  -v $(pwd)/pforge.yaml:/pforge.yaml \
  ghcr.io/paiml/pforge:latest \
  serve
```

**Docker Compose**:
```bash
# Clone repository
git clone https://github.com/paiml/pforge.git
cd pforge

# Start services
docker-compose up -d

# View logs
docker-compose logs -f
```

### 5. From Source

**Prerequisites**: Rust 1.75+, Git

```bash
# Clone repository
git clone https://github.com/paiml/pforge.git
cd pforge

# Build release binary
cargo build --release

# Install
cargo install --path crates/pforge-cli

# Or copy binary
sudo cp target/release/pforge /usr/local/bin/
```

---

## Platform-Specific Instructions

### Linux

#### Debian/Ubuntu
```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install via cargo
cargo install pforge-cli

# Or use install script
curl -sSL https://raw.githubusercontent.com/paiml/pforge/main/install.sh | bash
```

#### Fedora/RHEL
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install pforge
cargo install pforge-cli
```

#### Arch Linux
```bash
# Install from AUR (when available)
yay -S pforge

# Or via cargo
cargo install pforge-cli
```

### macOS

#### Homebrew (Recommended)
```bash
brew tap paiml/pforge
brew install pforge
```

#### Cargo
```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install pforge
cargo install pforge-cli
```

### Windows

#### Cargo (Recommended)
```powershell
# Install Rust from https://rustup.rs/

# Install pforge
cargo install pforge-cli
```

#### Binary Download
```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/paiml/pforge/releases/latest/download/pforge-windows-amd64.zip" -OutFile pforge.zip
Expand-Archive pforge.zip
Move-Item pforge\pforge.exe C:\Windows\System32\
```

---

## Post-Installation

### Shell Completions

#### Bash
```bash
pforge completions bash > ~/.local/share/bash-completion/completions/pforge
```

#### Zsh
```bash
pforge completions zsh > ~/.zfunc/_pforge
```

#### Fish
```bash
pforge completions fish > ~/.config/fish/completions/pforge.fish
```

### Verify Installation

```bash
# Check version
pforge --version

# Run help
pforge --help

# Create test server
pforge new test-server
cd test-server
pforge serve
```

---

## Updating

### Cargo
```bash
cargo install pforge-cli --force
```

### Homebrew
```bash
brew update
brew upgrade pforge
```

### Docker
```bash
docker pull ghcr.io/paiml/pforge:latest
```

---

## Troubleshooting

### Command Not Found

**Problem**: `pforge: command not found`

**Solution**: Add cargo bin to PATH
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc
```

### Build Failures

**Problem**: Compilation errors

**Solution**: Update Rust
```bash
rustup update stable
cargo install pforge-cli --force
```

### Permission Denied

**Problem**: Cannot install to /usr/local/bin

**Solution**: Use sudo or install to user directory
```bash
# Option 1: Use sudo
sudo cargo install pforge-cli --root /usr/local

# Option 2: Install to user directory
cargo install pforge-cli
# Binary will be in ~/.cargo/bin/
```

---

## Uninstallation

### Cargo
```bash
cargo uninstall pforge-cli
```

### Homebrew
```bash
brew uninstall pforge
brew untap paiml/pforge
```

### Binary
```bash
sudo rm /usr/local/bin/pforge
```

### Docker
```bash
docker rmi ghcr.io/paiml/pforge:latest
```

---

## Next Steps

After installation:

1. **Quick Start**:
   ```bash
   pforge new my-server
   cd my-server
   pforge serve
   ```

2. **Read Documentation**:
   - [User Guide](./USER_GUIDE.md)
   - [Examples](./examples/)
   - [Architecture](./ARCHITECTURE.md)

3. **Join Community**:
   - GitHub: https://github.com/paiml/pforge
   - Issues: https://github.com/paiml/pforge/issues

---

**Last Updated**: 2025-10-03
**pforge Version**: 0.1.0
