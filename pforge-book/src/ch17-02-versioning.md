# Version Management

Semantic versioning is the contract between you and your users. In the Rust ecosystem, version numbers communicate compatibility guarantees. This chapter covers version management for multi-crate workspaces like pforge.

## Semantic Versioning Basics

Semantic versioning (semver) uses three numbers: MAJOR.MINOR.PATCH

```
0.1.0
│ │ │
│ │ └─ PATCH: Bug fixes, no API changes
│ └─── MINOR: New features, backward compatible
└───── MAJOR: Breaking changes
```

### Version Increment Rules

Increment:
- **PATCH** (0.1.0 → 0.1.1): Bug fixes, documentation, internal optimizations
- **MINOR** (0.1.0 → 0.2.0): New features, new public APIs, deprecations
- **MAJOR** (0.1.0 → 1.0.0): Breaking changes, removed APIs, incompatible changes

### The 0.x Special Case

Versions before 1.0.0 have relaxed rules:

**For 0.y.z**:
- Increment **y** (minor) for breaking changes
- Increment **z** (patch) for all other changes

This acknowledges that pre-1.0 APIs are unstable.

**pforge uses 0.1.0** because:
- The framework is production-ready but evolving
- We reserve the right to make breaking changes
- Version 1.0.0 will signal API stability

### When to Release 1.0.0

Release 1.0.0 when:
- API is stable and well-tested
- No planned breaking changes
- Production deployments exist
- You commit to backward compatibility

For pforge, 1.0.0 will mean:
- MCP server schema is stable
- Core abstractions (Handler, Registry) won't change
- YAML configuration is locked
- Quality gates are production-proven

## Version Compatibility in Rust

Cargo uses semver to resolve dependencies.

### Caret Requirements (Default)

```toml
serde = "1.0"
```

Expands to: `>=1.0.0, <2.0.0`

Allows:
- 1.0.0 ✓
- 1.0.108 ✓
- 1.15.2 ✓
- 2.0.0 ✗ (breaking change)

This is **default and recommended** for libraries.

### Tilde Requirements

```toml
serde = "~1.0.100"
```

Expands to: `>=1.0.100, <1.1.0`

More restrictive - only allows patch updates.

### Exact Requirements

```toml
serde = "=1.0.100"
```

Exactly version 1.0.100, no other version.

**Avoid in libraries** - too restrictive, causes dependency conflicts.

### Wildcard Requirements

```toml
serde = "1.*"
```

Expands to: `>=1.0.0, <2.0.0`

Same as caret, but less clear. Use caret instead.

### Version Selection Strategy

**For libraries (like pforge-config)**:
- Use caret: `"1.0"`
- Allows users to upgrade dependencies
- Prevents dependency hell

**For binaries (like pforge-cli)**:
- Use caret: `"1.0"`
- Lock with `Cargo.lock` for reproducibility
- Commit `Cargo.lock` to repository

## Workspace Version Management

pforge uses workspace-level version management for consistency.

### Unified Versioning Strategy

**All pforge crates share the same version number**: 0.1.0

**Benefits**:
- Simple to understand: "pforge 0.1.0" refers to all crates
- Easy to document: one version per release
- Guaranteed compatibility: all crates from same release work together
- Simplified testing: test matrix doesn't explode

**Drawbacks**:
- Publish all crates even if some unchanged
- Version numbers jump (config might go 0.1.0 → 0.3.0 without changes)

**Alternative**: Independent versioning (each crate has own version). More complex but allows granular releases.

### Implementing Workspace Versions

In workspace root `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0"
```

Each crate inherits:

```toml
[package]
name = "pforge-config"
version.workspace = true
```

### Updating All Versions

To bump version across workspace:

```bash
# Edit workspace Cargo.toml
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# Update Cargo.lock
cargo update -w

# Verify
grep -r "version.*0.2.0" Cargo.toml
```

### Version Bumping Script

Automate with a script:

```bash
#!/bin/bash
# scripts/bump-version.sh

set -e

CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d '"' -f 2)
echo "Current version: $CURRENT_VERSION"
echo "Enter new version:"
read NEW_VERSION

# Validate semver format
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Version must be in format X.Y.Z"
    exit 1
fi

# Update workspace version
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update Cargo.lock
cargo update -w

# Update internal dependency versions in workspace dependencies
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/g" Cargo.toml

echo "Version bumped to $NEW_VERSION"
echo "Don't forget to:"
echo "  1. Update CHANGELOG.md"
echo "  2. Run: cargo test --all"
echo "  3. Commit changes"
echo "  4. Create git tag: git tag -a v$NEW_VERSION"
```

Run it:

```bash
./scripts/bump-version.sh
```

Example session:

```
Current version: 0.1.0
Enter new version:
0.2.0
Version bumped to 0.2.0
Don't forget to:
  1. Update CHANGELOG.md
  2. Run: cargo test --all
  3. Commit changes
  4. Create git tag: git tag -a v0.2.0
```

## Internal Dependency Versions

Workspace crates depending on each other need careful version management.

### The Problem

When `pforge-runtime` depends on `pforge-config`:

```toml
# In pforge-runtime/Cargo.toml
[dependencies]
pforge-config = { path = "../pforge-config", version = "0.1.0" }
```

**After version bump to 0.2.0**, this is now wrong. Runtime 0.2.0 still requires config 0.1.0.

### The Solution: Workspace Dependencies

Define once in workspace root:

```toml
[workspace.dependencies]
pforge-config = { path = "crates/pforge-config", version = "0.1.0" }
pforge-macro = { path = "crates/pforge-macro", version = "0.1.0" }
pforge-runtime = { path = "crates/pforge-runtime", version = "0.1.0" }
pforge-codegen = { path = "crates/pforge-codegen", version = "0.1.0" }
```

Crates reference with:

```toml
[dependencies]
pforge-config = { workspace = true }
```

**When you bump workspace version to 0.2.0**, update once in workspace dependencies section:

```toml
[workspace.dependencies]
pforge-config = { path = "crates/pforge-config", version = "0.2.0" }
pforge-macro = { path = "crates/pforge-macro", version = "0.2.0" }
pforge-runtime = { path = "crates/pforge-runtime", version = "0.2.0" }
pforge-codegen = { path = "crates/pforge-codegen", version = "0.2.0" }
```

All crates automatically use new version.

### Version Compatibility Between Internal Crates

For unified versioning:

```toml
# All internal deps use exact workspace version
pforge-config = { workspace = true }  # Resolves to "0.2.0"
```

For independent versioning:

```toml
# Allow compatible versions
pforge-config = { version = "0.2", path = "../pforge-config" }  # >=0.2.0, <0.3.0
```

**pforge uses unified versioning** for simplicity.

## Changelog Management

A CHANGELOG documents what changed between versions.

### CHANGELOG.md Structure

Follow "Keep a Changelog" format (https://keepachangelog.com):

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Feature X for doing Y

### Changed
- Refactored Z for performance

### Fixed
- Bug in handler dispatch

## [0.2.0] - 2025-02-15

### Added
- HTTP tool type support
- Middleware system for request/response transformation
- State persistence with sled backend

### Changed
- BREAKING: Renamed `ToolDefinition` to `ToolDef`
- Improved error messages with context

### Fixed
- Template files not included in pforge-cli package (#42)
- Race condition in handler registry

## [0.1.0] - 2025-01-10

### Added
- Initial release
- Native, CLI, and Pipeline tool types
- YAML configuration parsing
- Code generation from YAML to Rust
- Quality gates with PMAT integration
- Comprehensive test suite
```

### Changelog Categories

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be-removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Vulnerability fixes

### Marking Breaking Changes

Prefix with **BREAKING**:

```markdown
### Changed
- BREAKING: Renamed `ToolDefinition` to `ToolDef`
- BREAKING: Handler trait now requires `async fn execute`
```

Makes breaking changes obvious to users.

### Unreleased Section

Accumulate changes in `[Unreleased]` during development:

```markdown
## [Unreleased]

### Added
- WebSocket transport support
- Prometheus metrics

### Fixed
- Memory leak in long-running servers
```

On release, move to versioned section:

```markdown
## [Unreleased]

## [0.3.0] - 2025-03-20

### Added
- WebSocket transport support
- Prometheus metrics

### Fixed
- Memory leak in long-running servers
```

## Git Tags and Releases

Tag each release for reproducibility.

### Creating Version Tags

After bumping version and updating changelog:

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release version 0.2.0"

# Push tag to remote
git push origin v0.2.0
```

### Annotated vs Lightweight Tags

**Annotated** (recommended):

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
```

Includes tagger info, date, message.

**Lightweight**:

```bash
git tag v0.2.0
```

Just a pointer to commit. Use annotated for releases.

### Tag Naming Convention

Use `v` prefix: `v0.1.0`, `v0.2.0`, `v1.0.0`

**pforge convention**: `v{major}.{minor}.{patch}`

### Listing Tags

```bash
# List all tags
git tag

# List with messages
git tag -n

# List specific pattern
git tag -l "v0.*"
```

### Checking Out a Tag

Users can check out specific version:

```bash
git clone https://github.com/paiml/pforge
cd pforge
git checkout v0.1.0
cargo build
```

### Deleting Tags

If you tagged the wrong commit:

```bash
# Delete local tag
git tag -d v0.2.0

# Delete remote tag
git push --delete origin v0.2.0
```

Then create correct tag.

## Version Yanking

crates.io allows "yanking" versions - prevents new users from depending on them, but doesn't break existing users.

### When to Yank

Yank a version if:
- Critical security vulnerability
- Data corruption bug
- Completely broken functionality
- Published by mistake

**Don't yank for**:
- Minor bugs (publish patch instead)
- Deprecation (use proper deprecation)
- Regret about API design (breaking changes go in next major version)

### How to Yank

```bash
cargo yank --version 0.1.0
```

Output:

```
    Updating crates.io index
       Yank pforge-config@0.1.0
```

### Un-Yanking

Made a mistake yanking?

```bash
cargo yank --version 0.1.0 --undo
```

### Effect of Yanking

**Yanked versions**:
- Don't appear in default search results on crates.io
- Can't be specified in new `Cargo.toml` files (cargo will error)
- Still work for existing `Cargo.lock` files
- Still visible on crates.io with "yanked" label

**Use case**: pforge 0.1.0 had template bug. We:
1. Published 0.1.1 with fix
2. Yanked 0.1.0
3. New users get 0.1.1, existing users unaffected

## Pre-Release Versions

For alpha, beta, or release candidate versions, use pre-release identifiers.

### Pre-Release Format

```
1.0.0-alpha
1.0.0-alpha.1
1.0.0-beta
1.0.0-beta.2
1.0.0-rc.1
1.0.0
```

Semver ordering:
```
1.0.0-alpha < 1.0.0-alpha.1 < 1.0.0-beta < 1.0.0-rc.1 < 1.0.0
```

### Publishing Pre-Releases

```toml
[package]
version = "1.0.0-alpha.1"
```

```bash
cargo publish
```

Users must opt in:

```toml
[dependencies]
pforge-config = "1.0.0-alpha.1"  # Exact version
```

Or:

```toml
pforge-config = ">=1.0.0-alpha, <1.0.0"
```

### When to Use Pre-Releases

- **alpha**: Early testing, expect bugs, API may change
- **beta**: Feature-complete, polishing, API frozen
- **rc** (release candidate): Final testing before stable

**pforge strategy**: Once 1.0.0 is near:
1. Publish 1.0.0-beta.1
2. Solicit feedback
3. Publish 1.0.0-rc.1 after fixes
4. Publish 1.0.0 if RC is stable

## Version Strategy for Multi-Crate Publishing

Publishing multiple crates requires version coordination.

### pforge's Version Strategy

**All crates share version**: 0.1.0 → 0.2.0 for all

**Publishing order** (dependency-first):
1. pforge-config 0.2.0
2. pforge-macro 0.2.0 (parallel with config)
3. pforge-runtime 0.2.0 (depends on config)
4. pforge-codegen 0.2.0 (depends on config)
5. pforge-cli 0.2.0 (depends on all)

**After each publication**, verify on crates.io before continuing.

### Handling Version Mismatches

**Problem**: pforge-runtime 0.2.0 published, but pforge-config 0.2.0 isn't on crates.io yet.

**Error**:

```
error: no matching package named `pforge-config` found
location searched: registry `crates-io`
required by package `pforge-runtime v0.2.0`
```

**Solution**: Wait for pforge-config 0.2.0 to be available. crates.io processing takes 1-2 minutes.

### Version Skew Prevention

**Use exact versions for internal dependencies**:

```toml
[workspace.dependencies]
pforge-config = { path = "crates/pforge-config", version = "=0.2.0" }
```

The `=` ensures runtime 0.2.0 uses exactly config 0.2.0, not 0.2.1.

**Trade-off**: Stricter compatibility, but requires republishing dependents for patches.

**pforge uses caret** (`version = "0.2.0"` which means `>=0.2.0, <0.3.0`) because we do unified releases anyway.

## CHANGELOG Template

```markdown
# Changelog

All notable changes to pforge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security

## [0.1.0] - 2025-01-10

Initial release of pforge.

### Added
- **pforge-config**: YAML configuration parsing with schema validation
- **pforge-macro**: Procedural macros for handler generation
- **pforge-runtime**: Core runtime with handler registry and dispatch
- **pforge-codegen**: Code generation from YAML to Rust
- **pforge-cli**: Command-line interface (new, build, serve, dev, test)
- Native tool type: Zero-cost Rust handlers
- CLI tool type: Wrapper for command-line tools with streaming
- Pipeline tool type: Composable tool chains
- Quality gates: PMAT integration with pre-commit hooks
- Test suite: Unit, integration, property-based, mutation tests
- Documentation: Comprehensive specification and examples
- Examples: hello-world, calculator, pmat-server
- Performance: <1μs dispatch, <100ms cold start
- EXTREME TDD methodology: 5-minute cycles with quality enforcement

### Performance
- Tool dispatch (hot): < 1μs
- Cold start: < 100ms
- Sequential throughput: > 100K req/s
- Concurrent throughput (8-core): > 500K req/s
- Memory baseline: < 512KB

### Quality Metrics
- Test coverage: 85%
- Mutation score: 92%
- Technical Debt Grade: 0.82
- Cyclomatic complexity: Max 15 (target ≤20)
- Zero SATD comments
- Zero unwrap() in production code

[Unreleased]: https://github.com/paiml/pforge/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/paiml/pforge/releases/tag/v0.1.0
```

## Release Checklist

Before publishing a new version:

- [ ] Run full test suite: `cargo test --all`
- [ ] Run quality gates: `make quality-gate`
- [ ] Update version in `Cargo.toml` workspace section
- [ ] Update version in workspace dependencies
- [ ] Run `cargo update -w`
- [ ] Update CHANGELOG.md (move Unreleased to version section)
- [ ] Update documentation if needed
- [ ] Run `cargo doc --no-deps` to verify
- [ ] Commit changes: `git commit -m "Bump version to X.Y.Z"`
- [ ] Create git tag: `git tag -a vX.Y.Z -m "Release version X.Y.Z"`
- [ ] Push commits: `git push origin main`
- [ ] Push tags: `git push origin vX.Y.Z`
- [ ] Publish crates in dependency order
- [ ] Verify each publication on crates.io
- [ ] Test installation: `cargo install pforge-cli --force`
- [ ] Create GitHub release with CHANGELOG excerpt
- [ ] Announce release (Twitter, Reddit, Discord, etc.)

## Summary

Effective version management requires:

1. **Semantic versioning**: MAJOR.MINOR.PATCH with clear rules
2. **Workspace versions**: Unified versioning for consistency
3. **Internal dependencies**: Use workspace dependencies with versions
4. **Changelog**: Document every change with "Keep a Changelog" format
5. **Git tags**: Tag releases for reproducibility
6. **Yanking**: Use sparingly for critical issues
7. **Pre-releases**: alpha/beta/rc for testing before stable
8. **Coordination**: Publish in dependency order, verify each step

pforge's version strategy:
- Unified 0.x versioning across all crates
- Workspace-level version management
- Dependency-first publishing order
- Comprehensive CHANGELOG with breaking change markers
- Git tags for every release

Version 1.0.0 will signal API stability and production readiness.

---

**Next**: [Documentation](ch17-03-documentation.md)
