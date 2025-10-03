#!/bin/bash
# pforge installation script
# Supports: Linux (x86_64, ARM64), macOS (x86_64, ARM64)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="paiml/pforge"
BINARY_NAME="pforge"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   pforge Installation Script          ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        *)
            echo -e "${RED}Error: Unsupported OS: $os${NC}"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac

    echo -e "${GREEN}✓${NC} Detected platform: $OS-$ARCH"
}

# Get latest version from GitHub
get_latest_version() {
    VERSION=$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    if [ -z "$VERSION" ]; then
        echo -e "${YELLOW}Warning: Could not detect latest version, using v0.1.0${NC}"
        VERSION="v0.1.0"
    fi
    echo -e "${GREEN}✓${NC} Latest version: $VERSION"
}

# Download binary
download_binary() {
    local download_url="https://github.com/$REPO/releases/download/$VERSION/pforge-${OS}-${ARCH}"

    if [ "$OS" = "linux" ]; then
        download_url="${download_url}.tar.gz"
    elif [ "$OS" = "macos" ]; then
        download_url="${download_url}.tar.gz"
    fi

    echo -e "${GREEN}→${NC} Downloading from: $download_url"

    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"

    if curl -sSL "$download_url" -o pforge.tar.gz; then
        echo -e "${GREEN}✓${NC} Downloaded successfully"
    else
        echo -e "${RED}Error: Download failed${NC}"
        echo -e "${YELLOW}Tip: Install via cargo: cargo install pforge-cli${NC}"
        exit 1
    fi

    # Extract
    tar xzf pforge.tar.gz
    chmod +x pforge
}

# Install binary
install_binary() {
    echo -e "${GREEN}→${NC} Installing to $INSTALL_DIR"

    if [ -w "$INSTALL_DIR" ]; then
        mv pforge "$INSTALL_DIR/"
    else
        echo -e "${YELLOW}Note: Requires sudo for installation to $INSTALL_DIR${NC}"
        sudo mv pforge "$INSTALL_DIR/"
    fi

    echo -e "${GREEN}✓${NC} Installed successfully"
}

# Verify installation
verify_installation() {
    if command -v pforge &> /dev/null; then
        local installed_version=$(pforge --version | head -1)
        echo -e "${GREEN}✓${NC} Installation verified: $installed_version"
    else
        echo -e "${RED}Error: Installation verification failed${NC}"
        exit 1
    fi
}

# Main installation flow
main() {
    detect_platform
    get_latest_version
    download_binary
    install_binary
    verify_installation

    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   Installation Complete!              ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Quick start:"
    echo "  pforge new my-server"
    echo "  cd my-server"
    echo "  pforge serve"
    echo ""
    echo "Documentation:"
    echo "  https://github.com/paiml/pforge"
}

main
