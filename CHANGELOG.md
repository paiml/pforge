# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-08-20

**Fixed**

- **`pforge serve` no longer advertises tools it cannot call.** A `type: native`
  tool's handler is compiled into a *server binary*; the generic `pforge serve`
  has no knowledge of it. The server registered nothing for such tools yet added
  an MCP adapter for them anyway, so `tools/list` returned `hello` while
  `tools/call` answered `-32603 Tool not found: hello`, and `inputSchema.properties`
  was empty because there was no handler type to derive a schema from. `run()`
  now refuses to start, naming the offending tools and the remedy. (#12, #13)
- **`pforge --version` works.** clap's derive emits a version flag only when
  declared, and it never was — so nothing could identify the binary, including
  forjar's `cargo package` resource, which is how every managed tool in the fleet
  is verified. (#9)
- **`--no-default-features` builds.** `sse`, `websocket` and `http-handlers` were
  declared optional and used unconditionally, so they were optional in name only
  and a consumer avoiding reqwest/TLS got a compile error instead of a smaller
  build. Transports that are compiled out now fail at construction with a message
  naming the missing feature, rather than vanishing or panicking. (#10)
- **The fuzzing lane runs all three targets.** `fuzz_handler_dispatch` had failed
  to *build* since `HandlerRegistry::get` was removed, and `continue-on-error`
  turned that into a green tick every day. Building is now a hard gate, separate
  from findings. (#11)

**Added**

- `[package.metadata.transports]` declaring the `cli` and `mcp` interfaces, with
  e2e suites that spawn the release binary rather than calling the library — a
  library-level suite cannot observe whether a transport is reachable from `main`.
- `scripts/dogfood-use.sh` + `scripts/mcp_probe.py`: the release gate that runs
  the workflow `pforge new` itself prints, and asserts the invariant an MCP client
  depends on — every name in `tools/list` must be callable via `tools/call` —
  from both sides.
- A CI `feature-matrix` over all 7 feature configurations, wired into `gate`.
  It targets `-p pforge-runtime`, not `--workspace`: cargo unifies features across
  a workspace build, so a workspace-level matrix passes with the bug present.

## [0.2.0] - 2026-08-16

**Breaking**: `pforge-runtime` re-exports `pmcp` types in its public API —
`create_transport() -> Result<Box<dyn Transport>>` is `pmcp::shared::Transport` —
so moving pmcp 1.8 -> 2.18 changes that signature for consumers. That is why this
is 0.2.0 and not 0.1.5.

### Changed

- **pmcp 1.8 -> 2.18.** Published `pforge-runtime 0.1.4` declared `pmcp: ^1.8`
  non-optionally, so every consumer was held ten releases back and could not opt
  out — and anyone wanting pmcp 2.x elsewhere in their tree compiled two
  incompatible copies of it. Consumers also missed pmcp #316, where a stdio
  server dropped responses to requests it had already accepted once the client
  closed stdin (the ordinary batch/one-shot shape). No source changes were needed
  across pforge's 18 pmcp call sites.
- **trueno-db -> aprender-db.** `trueno-db` was consolidated into the aprender
  monorepo (APR-MONO, 2026-06-12) and is no longer published standalone; the
  crates.io copy is 0.4.0, last touched 2026-04-07, against aprender-db 0.63.0.
  Note for anyone doing the same migration: the PACKAGE renamed, the CRATE did
  not. `aprender-db` declares `[lib] name = "trueno_db"`, so `use trueno_db::...`
  stays correct and changing it to `aprender_db` gives E0433 even though
  `cargo tree` shows the dependency present.
- Every other dependency updated — 214 packages moved, including thiserror 1->2,
  rand 0.8->0.10, reqwest 0.12->0.13, notify 6->8, criterion 0.5->0.8. Only two
  code changes were required: reqwest 0.13 moved `RequestBuilder::query` behind a
  cargo feature, and criterion 0.8 deprecated its `black_box` re-export.

### Fixed

- **`pforge serve` corrupted its own protocol stream.** On a stdio transport
  stdout IS the MCP channel, and `serve` opened by writing five `println!` lines
  into it ahead of the first JSON-RPC frame ("Starting pforge server...",
  the config path, the server name, the transport, the tool count). A tolerant
  client skips them; a strict one fails to parse and reports the server as
  broken. `pforge dev` had the same defect and delegates to `serve`, so its
  output landed in the stream too. Both now write progress to stderr, which is
  what `McpServer::run` already did.
- **RUSTSEC-2026-0098, -0099, -0204**: `rustls-webpki` 0.103.10 (name-constraint
  bypasses for URI and wildcard names) and `crossbeam-epoch` 0.9.18 (invalid
  pointer dereference). Both transitive, so a library consumer resolving their
  own tree was already getting patched versions; this binds the lockfile for
  `cargo install --locked pforge`.
- Coverage no longer runs `cargo llvm-cov nextest`, which spawns a process per
  test binary and produces a profraw explosion. `PROPTEST_CASES` and
  `QUICKCHECK_TESTS` are pinned so a green property run means the same amount of
  searching on every machine.

### Added

- **Tests that prove pforge is an MCP server.** The suite had 228 tests and none
  of them had exchanged a JSON-RPC frame with a running server — every one
  stopped a layer below the protocol. `e2e_test.rs::test_stdio_transport_config`
  is the clearest case: its name says end-to-end, its body deserializes YAML and
  asserts an enum value. `mcp_protocol_test.rs` spawns the real binary, speaks
  MCP over stdin/stdout and asserts on the bytes returned. It found the stdout
  bug above on its first honest run.

## [0.1.4] - 2024-12-06

### Fixed
- **BREAKING**: Fixed pmcp 1.8.6 compatibility in pforge-runtime (#1)
  - `ToolInfo` struct is now non-exhaustive in pmcp 1.8.6
  - Updated to use `ToolInfo::new()` constructor instead of struct literal
  - Bumped pmcp dependency from 1.6 to 1.8

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
