# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2024-12-06

### Added
- TTL support for TruenoKvStateManager with automatic key expiration
- Timeout configuration (`timeout_ms`) for CLI and HTTP handlers
- Pipeline handler registration with PipelineHandlerAdapter
- Schema generation for MCP tools from registry
- Pipeline code generation with full step configuration
- PMAT compliance configuration (`.pmat-metrics.toml`)
- Workspace-level lints configuration
- Clippy configuration (`.clippy.toml`) with disallowed methods
- Feature flags for optional dependencies

### Changed
- HttpHandler now accepts timeout_ms parameter for request timeouts
- PforgeToolAdapter now fetches actual schemas from registry
- Updated book documentation for trueno-db TTL and HTTP timeout features
- 228 tests passing with 90.70% coverage

## [0.1.2] - 2024-12-05

### Added
- MCP registry publishing automation
- Comprehensive MCP registry publishing documentation
- MCP registry verification queries

### Fixed
- CI failures with tool installs
- Tarpaulin timeout issues for slow compilation tests
- GitHub Actions workflow failures

### Changed
- Updated server.json to use latest MCP registry schema
- Added comprehensive crate documentation

## [0.1.1] - 2024-12-04

### Added
- Quality deep dive with 91% coverage
- A+ grade achievement
- EXTREME TDD methodology chapters

### Changed
- Bumped version for quality improvements release
- Updated ROADMAP for v0.1.1 release

## [0.1.0] - 2024-12-03

### Added
- Initial production-ready release
- Zero-boilerplate MCP server framework
- Declarative YAML configuration
- Native, CLI, HTTP, and Pipeline handler types
- Multi-transport support (stdio, SSE, WebSocket)
- Language bridges for Python, Go, and Node.js
- PMAT quality enforcement integration
- Comprehensive telemetry and observability infrastructure
- Package distribution infrastructure
- Example servers:
  - Hello World (minimal viable server)
  - Calculator (arithmetic operations)
  - REST API Proxy (HTTP handler example)
  - PMAT Analysis Server (code analysis integration)
  - Polyglot Multi-Language Server (bridge examples)
  - Production-Ready Full-Featured Server

### Performance
- Tool dispatch < 1μs (hot path)
- Cold start < 100ms
- Config parse < 10ms
- Memory baseline < 512KB
- Throughput > 100K req/s (sequential)
- Throughput > 500K req/s (concurrent, 8-core)

### Quality
- 80%+ test coverage
- TDG score: 96/100
- Cyclomatic complexity max: 9
- Zero SATD comments
- A+ PMAT grade

[Unreleased]: https://github.com/paiml/pforge/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/paiml/pforge/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/paiml/pforge/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/paiml/pforge/releases/tag/v0.1.0
