# pforge

**Declarative MCP Server Framework with Zero Boilerplate**

Build [Model Context Protocol](https://modelcontextprotocol.io) servers using pure YAML configuration - no boilerplate code required.

[![CI](https://github.com/paiml/pforge/workflows/CI/badge.svg)](https://github.com/paiml/pforge/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## ✨ Features

- 🎯 **Zero Boilerplate**: Define MCP servers in YAML, not code
- 🚀 **Type Safe**: Full Rust type safety with Serde + JsonSchema
- ⚡ **High Performance**: O(1) handler dispatch, async-first architecture
- 🔌 **Multiple Handler Types**:
  - **Native**: Pure Rust handlers with full type safety
  - **CLI**: Execute shell commands
  - **HTTP**: Proxy REST APIs with template parameters
  - **Pipeline**: Chain tools together
- 💾 **State Management**: Persistent (Sled) and in-memory backends
- 🔄 **Fault Tolerance**: Circuit breaker, retry, timeout built-in
- 🎨 **Middleware**: Composable request/response processing
- 📦 **Resources & Prompts**: URI templates and prompt interpolation

## 🚀 Quick Start

### Installation

\`\`\`bash
# From source (recommended for now)
git clone https://github.com/paiml/pforge
cd pforge
cargo install --path crates/pforge-cli

# Coming soon: cargo install pforge-cli
\`\`\`

### Create Your First Server

\`\`\`bash
# Create new project
pforge new my-server
cd my-server

# Edit pforge.yaml to define your tools
# (see examples below)

# Run server
pforge serve
\`\`\`

## 📝 Configuration Examples

### Native Handler (Rust)

\`\`\`yaml
forge:
  name: my-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: greet
    description: "Greet a person by name"
    handler:
      path: handlers::greet::say_hello
    params:
      name:
        type: string
        required: true
\`\`\`

### HTTP Handler (No Code!)

\`\`\`yaml
tools:
  - type: http
    name: get_user
    description: "Get GitHub user info"
    endpoint: "https://api.github.com/users/{{username}}"
    method: GET
    headers:
      User-Agent: "my-mcp-server"
\`\`\`

## 📚 Documentation

- **[User Guide](docs/USER_GUIDE.md)** - Complete usage documentation
- **[Architecture](docs/ARCHITECTURE.md)** - Technical deep-dive
- **[Examples](examples/)** - Working examples to learn from

## 🎯 Examples

### [Hello World](examples/hello-world/)
Minimal MCP server with native handler

### [REST API Proxy](examples/rest-api-proxy/)
HTTP handlers for GitHub API - zero code needed

## 📊 Status

**Current Version**: 0.1.0  
**Status**: ✅ Production Ready (Core Features)

- ✅ 55 tests passing (100%)
- ✅ Full CI/CD automation
- ✅ Comprehensive documentation
- ✅ Working examples

## 🤝 Contributing

Contributions welcome! Please read [CLAUDE.md](CLAUDE.md) for development guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file

---

**Built with ❤️ by Pragmatic AI Labs**

🤖 *Implemented with [Claude Code](https://claude.com/claude-code)*
