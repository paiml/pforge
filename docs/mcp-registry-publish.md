# Publishing pforge to MCP Registry

This guide covers publishing pforge to the [Model Context Protocol (MCP) Registry](https://registry.modelcontextprotocol.io/), making it discoverable to MCP clients.

## Overview

The MCP Registry is a centralized directory for MCP servers, similar to an app store. Publishing pforge to the registry allows users to:

- Discover pforge through MCP clients
- Install pforge via standardized commands
- View server metadata and capabilities
- Access documentation and examples

## Automated Publishing Process

Publishing to the MCP registry is automated via GitHub Actions. When you create a tag with the `mcp-v*` pattern, the workflow:

1. ✅ Runs all tests
2. ✅ Builds release binaries
3. ✅ Publishes to crates.io (all workspace crates)
4. ✅ Authenticates to MCP registry via GitHub OIDC
5. ✅ Publishes server metadata to MCP registry

## Prerequisites

### For Repository Maintainers

1. **CARGO_TOKEN Secret** (Required for crates.io publishing)
   - Go to https://github.com/paiml/pforge/settings/secrets/actions
   - Create secret named `CARGO_TOKEN`
   - Value: Your crates.io API token from https://crates.io/settings/tokens
   - Ensure token has publish permissions

2. **Organization Membership** (Required for MCP registry)
   - Must be a member of the `paiml` GitHub organization
   - Server namespace is `io.github.paiml/pforge`
   - GitHub OIDC authentication restricts publishing to org members

3. **Version Alignment**
   - Ensure `Cargo.toml` version matches intended release
   - Server.json version should match workspace version
   - Follow semantic versioning (MAJOR.MINOR.PATCH)

## Publishing Instructions

### Step 1: Prepare Release

```bash
# Ensure you're on main branch
git checkout main
git pull origin main

# Run quality gates
make quality-gate

# Verify all tests pass
cargo test --all --release

# Update version in Cargo.toml if needed
# (Workspace version is in root Cargo.toml)
```

### Step 2: Create MCP Release Tag

```bash
# Create MCP-specific release tag
# Format: mcp-vMAJOR.MINOR.PATCH
git tag mcp-v0.1.2 -m "MCP Registry release v0.1.2"

# Push tag to trigger workflow
git push origin mcp-v0.1.2
```

**Important:** Use the `mcp-v*` prefix to trigger the MCP publishing workflow. Regular `v*` tags trigger standard release workflows.

### Step 3: Monitor Workflow

1. Go to https://github.com/paiml/pforge/actions
2. Find "Publish to MCP Registry" workflow
3. Monitor the publishing steps:
   - Test execution
   - Release build
   - Crates.io publishing
   - MCP registry authentication
   - MCP registry publishing

### Step 4: Verify Publication

```bash
# Check MCP registry
curl "https://registry.modelcontextprotocol.io/v0/servers?search=pforge" | jq

# Verify server appears in results
# Expected: io.github.paiml/pforge
```

## Manual Publishing (Alternative)

If you need to publish manually:

```bash
# Download MCP publisher
curl -sL "https://github.com/modelcontextprotocol/registry/releases/download/v1.2.3/mcp-publisher_1.2.3_linux_amd64.tar.gz" | tar xz
chmod +x mcp-publisher

# Authenticate (requires paiml org membership)
./mcp-publisher login github

# Publish
./mcp-publisher publish
```

## Manual Workflow Trigger

You can also trigger the publishing workflow manually from the GitHub UI:

1. Go to https://github.com/paiml/pforge/actions/workflows/publish-mcp.yml
2. Click "Run workflow"
3. Select the branch (usually `main`)
4. Click "Run workflow"

This is useful for:
- Re-publishing after registry issues
- Testing the workflow
- Publishing without creating a tag

## Server Metadata Configuration

The server metadata is defined in `server.json`:

```json
{
  "$schema": "https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json",
  "name": "io.github.paiml/pforge",
  "description": "Zero-boilerplate MCP server framework with declarative YAML configuration",
  "repository": {
    "url": "https://github.com/paiml/pforge",
    "source": "github"
  },
  "version": "0.1.2",
  "installInstructions": "Install via Cargo: cargo install pforge-cli",
  "homepage": "https://github.com/paiml/pforge",
  "license": "MIT",
  "tags": [
    "framework",
    "codegen",
    "yaml",
    "declarative",
    "rust",
    "tdd"
  ]
}
```

### Key Fields

- **name**: Namespace format `io.github.{org}/{repo}`
- **description**: Must be ≤100 characters
- **version**: Should match workspace version
- **installInstructions**: How users install the server
- **tags**: Searchable keywords for discovery

## Troubleshooting

### Common Issues

**Issue: "You do not have permission to publish this server"**
- **Cause**: Not authenticated as paiml org member
- **Solution**: Ensure you're logged in with a GitHub account that's a member of the paiml organization

**Issue: "validation failed: description too long"**
- **Cause**: Description exceeds 100 characters
- **Solution**: Edit `server.json` and shorten the description field

**Issue: "server.json invalid: cannot unmarshal string into Repository"**
- **Cause**: Using old schema format
- **Solution**: Repository must be an object with `url` and `source` fields:
  ```json
  "repository": {
    "url": "https://github.com/paiml/pforge",
    "source": "github"
  }
  ```

**Issue: Crates.io publishing fails**
- **Cause**: Missing or invalid CARGO_TOKEN secret
- **Solution**: Add valid crates.io token to GitHub secrets
- **Note**: Workflow continues even if crates.io fails (uses `continue-on-error: true`)

**Issue: Tests fail during publishing**
- **Cause**: Code doesn't pass quality gates
- **Solution**: Run `make quality-gate` locally and fix issues before tagging

### Registry Schema Updates

The MCP registry schema is versioned. Current schema: `2025-09-29`

If the schema updates:

1. Check the new schema at: https://static.modelcontextprotocol.io/schemas/{version}/server.schema.json
2. Update `server.json` to match new schema
3. Test with `mcp-publisher publish` (dry-run if available)
4. Update this documentation with changes

## Version Management

### Semantic Versioning

pforge follows semantic versioning:

- **MAJOR**: Breaking changes to API or configuration format
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Release Process

1. Update `Cargo.toml` workspace version
2. Update `server.json` version to match
3. Update `CHANGELOG.md` with release notes
4. Run quality gates: `make quality-gate`
5. Create tag: `git tag mcp-v{version}`
6. Push tag: `git push origin mcp-v{version}`

### Tag Patterns

- `mcp-v*` - Triggers MCP registry publishing workflow
- `v*` - Triggers standard release workflow (crates.io only)

## Dependencies

### Required Tools (CI Environment)

- Rust toolchain (stable)
- Cargo with publish permissions
- mcp-publisher CLI (auto-downloaded by workflow)
- GitHub OIDC authentication (provided by Actions)

### Required Secrets

- `CARGO_TOKEN` - Crates.io API token for publishing

### Required Permissions

- GitHub Actions: `id-token: write` (for OIDC)
- GitHub Actions: `contents: read` (for checkout)

## Verification Queries

After publishing, verify pforge is in the registry:

### Search by name:
```bash
curl -s "https://registry.modelcontextprotocol.io/v0/servers?search=pforge" | jq '.'
```

### Get server directly (note: `/` must be URL-encoded as `%2F`):
```bash
curl -s "https://registry.modelcontextprotocol.io/v0/servers/io.github.paiml%2Fpforge" | jq '.'
```

### Expected response:
```json
{
  "server": {
    "name": "io.github.paiml/pforge",
    "description": "Zero-boilerplate MCP server framework with declarative YAML configuration",
    "version": "0.1.2",
    "repository": {
      "url": "https://github.com/paiml/pforge",
      "source": "github"
    }
  },
  "_meta": {
    "io.modelcontextprotocol.registry/official": {
      "status": "active",
      "publishedAt": "2025-10-04T14:21:46.420973Z",
      "isLatest": true
    }
  }
}
```

## Resources

- **MCP Registry**: https://registry.modelcontextprotocol.io/
- **Registry API Docs**: https://registry.modelcontextprotocol.io/docs
- **Registry OpenAPI**: https://registry.modelcontextprotocol.io/openapi.yaml
- **Publishing Guide**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/publish-server.md
- **GitHub Actions**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/github-actions.md
- **Schema**: https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json

## Support

For issues with:
- **pforge publishing**: Open issue at https://github.com/paiml/pforge/issues
- **MCP registry**: GitHub Discussions at https://github.com/modelcontextprotocol/registry/discussions
- **Registry schema**: Discord #registry-dev channel

---

*Last updated: 2025-10-04*
