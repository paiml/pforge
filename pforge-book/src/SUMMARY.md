# Summary

[pforge: EXTREME TDD for MCP Servers](title-page.md)
[Introduction](introduction.md)

## Part I: Understanding pforge

- [Chapter 1: pforge vs pmcp (rust-mcp-sdk)](ch01-00-pforge-vs-pmcp.md)
    - [When to Use pforge](ch01-01-when-pforge.md)
    - [When to Use pmcp](ch01-02-when-pmcp.md)
    - [Side-by-Side Comparison](ch01-03-comparison.md)
    - [Migration Between Them](ch01-04-migration.md)
    - [Architecture: How pforge Uses pmcp](ch01-05-architecture-pmcp.md)

- [Chapter 2: Quick Start](ch02-00-quick-start.md)
    - [Installation](ch02-01-installation.md)
    - [Your First Server (5 Minutes)](ch02-02-first-server.md)
    - [Testing Your Server](ch02-03-testing.md)

## Part II: Working Examples (All Tested)

- [Chapter 3: Calculator Server](ch03-00-calculator.md)
    - [YAML Configuration](ch03-01-yaml-config.md)
    - [Rust Handler Implementation](ch03-02-handler.md)
    - [Unit Tests](ch03-03-tests.md)
    - [Running the Server](ch03-04-running.md)

- [Chapter 4: File Operations Server](ch04-00-file-ops.md)
    - [CLI Tool Wrappers](ch04-01-cli-wrappers.md)
    - [Streaming Output](ch04-02-streaming.md)
    - [Integration Tests](ch04-03-integration-tests.md)

- [Chapter 5: GitHub API Server](ch05-00-github-api.md)
    - [HTTP Tool Configuration](ch05-01-http-config.md)
    - [Authentication](ch05-02-authentication.md)
    - [Error Handling](ch05-03-error-handling.md)

- [Chapter 6: Data Pipeline Server](ch06-00-data-pipeline.md)
    - [Pipeline Composition](ch06-01-composition.md)
    - [Conditional Execution](ch06-02-conditionals.md)
    - [State Management](ch06-03-state.md)

## Part III: EXTREME TDD Methodology

- [Chapter 7: The 5-Minute TDD Cycle](ch07-00-five-minute-cycle.md)
    - [RED: Write Failing Test](ch07-01-red.md)
    - [GREEN: Minimum Code](ch07-02-green.md)
    - [REFACTOR: Clean Up](ch07-03-refactor.md)
    - [COMMIT: Quality Gates](ch07-04-commit.md)

- [Chapter 8: Quality Gates](ch08-00-quality-gates.md)
    - [Pre-Commit Hooks](ch08-01-pre-commit.md)
    - [PMAT Integration](ch08-02-pmat.md)
    - [Complexity Limits](ch08-03-complexity.md)
    - [Coverage Requirements](ch08-04-coverage.md)

- [Chapter 9: Testing Strategies](ch09-00-testing-strategies.md)
    - [Unit Testing](ch09-01-unit-testing.md)
    - [Integration Testing](ch09-02-integration-testing.md)
    - [Property-Based Testing](ch09-03-property-testing.md)
    - [Mutation Testing](ch09-04-mutation-testing.md)

## Part IV: Advanced Features

- [Chapter 10: State Management](ch10-00-state-management.md)
- [Chapter 11: Fault Tolerance](ch11-00-fault-tolerance.md)
- [Chapter 12: Middleware](ch12-00-middleware.md)
- [Chapter 13: Resources & Prompts](ch13-00-resources-prompts.md)

## Part V: Performance & Optimization

- [Chapter 14: Performance Targets](ch14-00-performance.md)
- [Chapter 15: Benchmarking](ch15-00-benchmarking.md)
- [Chapter 16: Code Generation](ch16-00-codegen.md)

## Part VI: Production Deployment

- [Chapter 17: Publishing to Crates.io](ch17-00-publishing-crates.md)
    - [Preparing Your Crate](ch17-01-preparing.md)
    - [Version Management](ch17-02-versioning.md)
    - [Documentation](ch17-03-documentation.md)
    - [Publishing Process](ch17-04-publishing.md)

- [Chapter 18: CI/CD Pipeline](ch18-00-cicd.md)
- [Chapter 19: Multi-Language Bridges](ch19-00-bridges.md)
    - [Python Bridge with EXTREME TDD](ch19-01-python-tdd.md)
    - [Go Bridge with EXTREME TDD](ch19-02-go-tdd.md)

## Appendices

- [Appendix A: Complete Configuration Reference](appendix-a-config-reference.md)
- [Appendix B: API Documentation](appendix-b-api-docs.md)
- [Appendix C: Troubleshooting](appendix-c-troubleshooting.md)
- [Appendix D: Contributing](appendix-d-contributing.md)
