# Production MCP Server - pforge Example

**Complete production-ready MCP server showcasing all pforge features.**

This comprehensive example demonstrates every pforge capability in a single, production-grade server:
- ✅ State management (persistent counters)
- ✅ Native handlers with complex logic
- ✅ CLI handlers with streaming
- ✅ HTTP handlers with authentication
- ✅ Pipeline workflows
- ✅ Resources (files, documentation)
- ✅ Prompts (AI assistance)
- ✅ Structured logging (tracing)
- ✅ Comprehensive error handling
- ✅ Production optimization

---

## Quick Start

**Prerequisites**: Rust 1.75+, Cargo

```bash
cd examples/production-server
cargo run --release

# Expected output:
# ╔═══════════════════════════════════════╗
# ║   Production MCP Server v1.0.0        ║
# ║   Full Feature Showcase               ║
# ║   Powered by pforge v0.1.0            ║
# ╚═══════════════════════════════════════╝
#
# Production Features:
#   ✓ State management (memory backend)
#   ✓ Native handlers with validation
#   ✓ CLI handlers with streaming
#   ✓ HTTP handlers with auth
#   ✓ Pipeline workflows
#   ✓ Resources and prompts
#   ✓ Structured logging
#   ✓ Comprehensive error handling
```

---

## Features Demonstrated

### 1. State Management
- **counter_increment**: Persistent counters with TTL
- Memory backend (10MB capacity, 1-hour TTL)
- Thread-safe state access
- Automatic expiration

### 2. Native Handlers
- **counter_increment**: Stateful counter with state persistence
- **data_processor**: Complex data validation and formatting

### 3. CLI Handlers
- **log_stream**: Real-time log streaming with journalctl

### 4. HTTP Handlers
- **api_fetch**: GitHub API integration with auth headers

### 5. Pipeline Workflows
- **full_workflow**: Multi-step pipeline combining all tools

### 6. Resources
- Server documentation (README.md)
- Configuration file (config.json)

### 7. Prompts
- generate_report: AI-assisted status reports
- troubleshoot: Error diagnosis assistance

### 8. Production Features
- Structured logging (tracing)
- Environment-based log levels
- Error handling with context
- Release optimization

---

## Tools Reference

### counter_increment
Stateful counter with persistence.

**Input**:
- `name` (string, required): Counter name
- `increment` (integer, optional, default: 1): Amount to increment

**Output**:
```json
{
  "name": "requests",
  "value": 42,
  "previous_value": 41,
  "increment": 1
}
```

### data_processor
Process and validate data structures.

**Input**:
- `data` (object, required): Data to process
- `format` (string, optional, default: "json"): Output format (json/yaml/toml)

**Output**:
```json
{
  "processed_data": "{...}",
  "format": "json",
  "size_bytes": 123,
  "validation": {
    "valid": true,
    "errors": [],
    "warnings": []
  }
}
```

### log_stream (CLI)
Stream system logs in real-time.

**Note**: Linux only, requires journalctl

### api_fetch (HTTP)
Fetch from GitHub API.

**Note**: Requires internet connection

### full_workflow (Pipeline)
Complete multi-tool workflow.

---

## Configuration Deep Dive

### State Backend
```yaml
state:
  backend: memory
  ttl_seconds: 3600  # 1 hour
  max_size: 10485760  # 10MB
```

**Production alternatives**:
- Switch to `sled` for disk persistence
- Configure Redis for distributed state

### Tool Timeouts
```yaml
tools:
  - name: counter_increment
    timeout_ms: 1000  # Fast operation

  - name: log_stream
    timeout_ms: 30000  # Long-running stream
```

### HTTP Authentication
```yaml
tools:
  - type: http
    headers:
      Authorization: "Bearer ${GITHUB_TOKEN}"
      User-Agent: "pforge-server/1.0"
```

---

## Production Deployment

### 1. Build for Production
```bash
cargo build --release
# Binary: target/release/production-server
```

### 2. Environment Configuration
```bash
# Logging level
export RUST_LOG=production_server=info

# API tokens (if needed)
export GITHUB_TOKEN=your_token_here

# Run server
./target/release/production-server
```

### 3. Systemd Service
```ini
[Unit]
Description=pforge Production MCP Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/production-server
WorkingDirectory=/opt/production-server
Environment=RUST_LOG=info
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 4. Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/production-server /usr/local/bin/
COPY pforge.yaml config.json /etc/production-server/
CMD ["production-server"]
```

---

## Monitoring & Observability

### Structured Logging
```bash
# Debug level
RUST_LOG=debug ./production-server

# JSON logs (for log aggregation)
RUST_LOG_FORMAT=json ./production-server
```

### Log Output
```
2025-10-03T12:00:00.123Z INFO production_server: Starting Production MCP Server v1.0.0
2025-10-03T12:00:00.456Z INFO production_server: Configuration loaded successfully
2025-10-03T12:00:00.789Z INFO production_server: State manager initialized: 10MB capacity, 1h TTL
```

### Metrics (Future)
```yaml
# Add to pforge.yaml
middleware:
  - type: metrics
    prometheus_port: 9090
```

---

## Testing

### Run Unit Tests
```bash
cargo test

# With logging
RUST_LOG=debug cargo test
```

### Integration Testing
```bash
# Test counter persistence
echo '{"name":"test","increment":5}' | \
  pforge-mcp-client counter_increment

# Test data processing
echo '{"data":{"key":"value"},"format":"yaml"}' | \
  pforge-mcp-client data_processor
```

---

## Troubleshooting

### journalctl Not Found
**Solution**: CLI handlers optional - disable in pforge.yaml or install systemd

### GitHub API Rate Limit
**Solution**: Add authentication token in headers

### State Not Persisting
**Solution**: Upgrade to sled backend for disk persistence

---

## Next Steps

1. **Customize for Your Use Case**:
   - Add domain-specific handlers
   - Configure production state backend
   - Set up monitoring

2. **Deploy to Production**:
   - Use release builds
   - Configure systemd/Docker
   - Set up log aggregation

3. **Scale Horizontally**:
   - Use Redis for shared state
   - Load balance with nginx
   - Deploy multiple instances

---

## Learn More

- [pforge User Guide](../../USER_GUIDE.md)
- [Architecture](../../ARCHITECTURE.md)
- [Other Examples](../)

---

**License**: MIT
**Version**: 1.0.0
