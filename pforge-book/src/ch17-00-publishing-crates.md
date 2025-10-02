# Publishing to Crates.io

Publishing your pforge crates to crates.io makes them available to the Rust ecosystem and allows users to install your MCP servers with a simple `cargo install` command. This chapter covers the complete publishing workflow based on pforge's real-world experience publishing five interconnected crates.

## Why Publish to Crates.io?

Publishing to crates.io provides several benefits:

1. **Easy Installation**: Users can install with `cargo install pforge-cli` instead of building from source
2. **Dependency Management**: Other crates can depend on your published crates with automatic version resolution
3. **Discoverability**: Your crates appear in searches on crates.io and docs.rs
4. **Documentation**: Automatic documentation generation and hosting on docs.rs
5. **Versioning**: Semantic versioning guarantees compatibility and upgrade paths
6. **Trust**: Published crates undergo community review and validation

## The pforge Publishing Story

pforge consists of five published crates that work together:

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `pforge-config` | Configuration parsing and validation | None (foundation) |
| `pforge-macro` | Procedural macros | None (independent) |
| `pforge-runtime` | Core runtime and handler registry | config |
| `pforge-codegen` | Code generation from YAML to Rust | config |
| `pforge-cli` | Command-line interface and templates | config, runtime, codegen |

This dependency chain means **publishing order matters critically**. You must publish foundation crates before crates that depend on them.

## Publishing Challenges We Encountered

When publishing pforge, we hit several real-world issues:

### 1. Rate Limiting

crates.io rate-limits new crate publications to prevent spam. Publishing five crates in rapid succession triggered:

```
error: failed to publish to crates.io

Caused by:
  the remote server responded with an error: too many crates published too quickly
```

**Solution**: Wait 10-15 minutes between publications, or publish over multiple days.

### 2. Missing Metadata

First publication attempt failed with:

```
error: missing required metadata fields:
  - description
  - keywords
  - categories
  - license
```

**Solution**: Add comprehensive metadata to `Cargo.toml` workspace section (covered in Chapter 17-01).

### 3. Template Files Not Included

The CLI crate initially failed to include template files needed for `pforge new`:

```
error: templates not found after installation
```

**Solution**: Add `include` field to `Cargo.toml`:

```toml
include = [
    "src/**/*",
    "templates/**/*",
    "Cargo.toml",
]
```

### 4. Version Specification Conflicts

Publishing `pforge-runtime` failed because it depended on `pforge-config = { path = "../pforge-config" }` without a version:

```
error: all dependencies must have version numbers for published crates
```

**Solution**: Use workspace dependencies with explicit versions (covered in Chapter 17-02).

### 5. Documentation Links Broken

docs.rs generation failed because README links used repository-relative paths:

```
warning: documentation link failed to resolve
```

**Solution**: Use absolute URLs in documentation or test with `cargo doc --no-deps`.

## The Publishing Workflow

Based on these experiences, here's the proven workflow:

### 1. Prepare All Crates (Chapter 17-01)
- Add required metadata
- Configure workspace inheritance
- Set up `include` fields
- Write comprehensive README files

### 2. Manage Versions (Chapter 17-02)
- Follow semantic versioning
- Update all internal dependencies
- Create version tags
- Update CHANGELOG

### 3. Write Documentation (Chapter 17-03)
- Add crate-level docs (`lib.rs`)
- Document all public APIs
- Create examples
- Test documentation builds

### 4. Publish in Order (Chapter 17-04)
- Test with `cargo publish --dry-run`
- Publish foundation crates first
- Wait for crates.io processing
- Verify each publication
- Continue up dependency chain

### 5. Post-Publication
- Test installation from crates.io
- Verify docs.rs generation
- Announce the release
- Monitor for issues

## The Dependency Chain

Understanding the dependency chain is crucial for successful publication:

```
pforge-config (no deps) ←─────┐
                              │
pforge-macro (no deps)        │
                              │
pforge-runtime (depends) ─────┘
       ↑
       │
pforge-codegen (depends)
       ↑
       │
pforge-cli (depends on runtime + codegen)
```

**Critical Rule**: Never publish a crate before its dependencies are available on crates.io.

## Publishing Order for pforge

The exact order we used:

1. **Day 1**: `pforge-config` and `pforge-macro` (independent, can be parallel)
2. **Day 1** (after 15 min): `pforge-runtime` (depends on config)
3. **Day 2**: `pforge-codegen` (depends on config)
4. **Day 2** (after 15 min): `pforge-cli` (depends on all three)

We spread publications across two days to avoid rate limiting and allow time for verification between steps.

## Verification Steps

After each publication:

### 1. Check crates.io

Visit `https://crates.io/crates/pforge-config` and verify:
- Version number is correct
- Description and keywords appear
- License is displayed
- Repository link works

### 2. Check docs.rs

Visit `https://docs.rs/pforge-config` and verify:
- Documentation builds successfully
- All modules are documented
- Examples render correctly
- Links work

### 3. Test Installation

On a clean machine or Docker container:

```bash
cargo install pforge-cli
pforge --version
pforge new test-project
```

This ensures the published crate actually works for end users.

## Rollback and Fixes

**Important**: crates.io is **append-only**. You cannot:
- Delete published versions
- Modify published crate contents
- Unpublish a version (only yank it)

If you publish with a bug:

### Option 1: Yank the Version

```bash
cargo yank --version 0.1.0
```

This prevents new projects from using the version but doesn't break existing users.

### Option 2: Publish a Patch

```bash
# Fix the bug
# Bump version to 0.1.1
cargo publish
```

The new version becomes the default, but the old version remains accessible.

## Pre-Publication Checklist

Before publishing ANY crate, verify:

- [ ] All tests pass: `cargo test --all`
- [ ] Quality gates pass: `make quality-gate`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] Dry run succeeds: `cargo publish --dry-run`
- [ ] Dependencies are published (for non-foundation crates)
- [ ] Version numbers are correct
- [ ] CHANGELOG is updated
- [ ] Git tags are created
- [ ] README is comprehensive
- [ ] Examples work

## Publishing Tools

Helpful tools for the publishing process:

```bash
# Check what will be included in the package
cargo package --list

# Create a .crate file without publishing
cargo package

# Inspect the .crate file
tar -tzf target/package/pforge-config-0.1.0.crate

# Dry run (doesn't actually publish)
cargo publish --dry-run

# Publish with dirty git tree (use cautiously)
cargo publish --allow-dirty
```

## Common Pitfalls

### 1. Publishing Without Testing

**Problem**: Rushing to publish without thorough testing.

**Solution**: Always run the pre-publication checklist. We found bugs in `pforge-cli` template handling only after attempting publication.

### 2. Incorrect Version Dependencies

**Problem**: Internal dependencies using `path` without `version`.

**Solution**: Use workspace dependencies with explicit versions:

```toml
pforge-config = { workspace = true }
```

### 3. Missing Files

**Problem**: Source files or resources not included in package.

**Solution**: Use `include` field or check with `cargo package --list`.

### 4. Platform-Specific Code

**Problem**: Code that only works on Linux but no platform guards.

**Solution**: Add `#[cfg(...)]` attributes and test on all platforms before publishing.

### 5. Large Crate Size

**Problem**: Accidentally including test data or build artifacts.

**Solution**: Use `.cargo-ignore` (similar to `.gitignore` but for cargo packages).

## Multi-Crate Workspace Tips

For workspaces like pforge with multiple publishable crates:

### 1. Shared Metadata

Define common metadata in `[workspace.package]`:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["Pragmatic AI Labs"]
repository = "https://github.com/paiml/pforge"
```

Each crate inherits with:

```toml
[package]
name = "pforge-config"
version.workspace = true
edition.workspace = true
license.workspace = true
```

### 2. Shared Dependencies

Define versions once in `[workspace.dependencies]`:

```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
pforge-config = { path = "crates/pforge-config", version = "0.1.0" }
```

Crates use with:

```toml
[dependencies]
serde = { workspace = true }
pforge-config = { workspace = true }
```

### 3. Version Bumping Script

Create a script to bump all versions simultaneously:

```bash
#!/bin/bash
NEW_VERSION=$1
sed -i "s/^version = .*/version = \"$NEW_VERSION\"/" Cargo.toml
for crate in crates/*/Cargo.toml; do
    # Versions are inherited, so this updates workspace version
    echo "Updated $crate"
done
cargo update -w
```

## Documentation Best Practices

Good documentation drives adoption:

### 1. Crate-Level Documentation

Add to `lib.rs`:

```rust
//! # pforge-config
//!
//! Configuration parsing and validation for pforge MCP servers.
//!
//! This crate provides the core configuration types and parsing logic
//! used by the pforge framework.
//!
//! ## Example
//!
//! ```rust
//! use pforge_config::ForgeConfig;
//!
//! let yaml = r#"
//! forge:
//!   name: my-server
//!   version: 0.1.0
//! "#;
//!
//! let config = ForgeConfig::from_yaml(yaml)?;
//! assert_eq!(config.name, "my-server");
//! ```
```

### 2. Module Documentation

Document each public module:

```rust
/// Tool definition types and validation.
///
/// This module contains the [`ToolDef`] enum and related types
/// for defining MCP tools declaratively.
pub mod tools;
```

### 3. Examples Directory

Add runnable examples in `examples/`:

```
crates/pforge-config/
├── examples/
│   ├── basic_config.rs
│   ├── validation.rs
│   └── advanced_features.rs
```

Users can run them with:

```bash
cargo run --example basic_config
```

## Chapter Summary

Publishing to crates.io requires careful preparation, strict ordering, and attention to detail. The key lessons from pforge's publishing experience:

1. **Metadata is mandatory**: Description, keywords, categories, license
2. **Order matters**: Publish dependencies before dependents
3. **Rate limits exist**: Space out publications by 10-15 minutes
4. **Include everything**: Templates, resources, documentation
5. **Test thoroughly**: Dry runs, package inspection, clean installs
6. **Document well**: Users rely on docs.rs
7. **Version carefully**: Semantic versioning is a contract
8. **No rollbacks**: You can't unpublish, only yank and patch

The next four chapters dive deep into each phase of the publishing process.

---

**Next**: [Preparing Your Crate](ch17-01-preparing.md)
