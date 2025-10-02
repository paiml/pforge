# Preparing Your Crate for Publication

Before publishing to crates.io, your crate needs proper metadata, documentation, and configuration. This chapter walks through preparing each pforge crate based on real-world experience.

## Required Metadata Fields

crates.io requires specific metadata in `Cargo.toml`. Missing any of these will cause publication to fail.

### Minimum Required Fields

```toml
[package]
name = "pforge-config"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Configuration parsing and validation for pforge MCP servers"
```

These five fields are **mandatory**. Attempting to publish without them produces:

```
error: failed to publish to crates.io

Caused by:
  missing required metadata fields: description, license
```

### Recommended Fields

For better discoverability and user experience, add:

```toml
[package]
# Required
name = "pforge-config"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Configuration parsing and validation for pforge MCP servers"

# Strongly recommended
repository = "https://github.com/paiml/pforge"
homepage = "https://github.com/paiml/pforge"
documentation = "https://docs.rs/pforge-config"
keywords = ["mcp", "config", "yaml", "codegen", "framework"]
categories = ["development-tools", "config", "parsing"]
authors = ["Pragmatic AI Labs"]
readme = "README.md"
```

Each field serves a specific purpose:

- **repository**: Link to source code (enables "Repository" button on crates.io)
- **homepage**: Project website (can be same as repository)
- **documentation**: Custom docs URL (defaults to docs.rs if omitted)
- **keywords**: Search terms (max 5, each max 20 chars)
- **categories**: Classification (from https://crates.io/categories)
- **authors**: Credit (can be organization or individuals)
- **readme**: README file path (relative to Cargo.toml)

## Workspace Metadata Pattern

For multi-crate workspaces like pforge, use workspace inheritance to avoid repetition.

### Workspace Root Configuration

In the root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/pforge-cli",
    "crates/pforge-runtime",
    "crates/pforge-codegen",
    "crates/pforge-config",
    "crates/pforge-macro",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/paiml/pforge"
authors = ["Pragmatic AI Labs"]
description = "Zero-boilerplate MCP server framework with EXTREME TDD methodology"
keywords = ["mcp", "codegen", "tdd", "framework", "declarative"]
categories = ["development-tools", "web-programming", "command-line-utilities"]
homepage = "https://github.com/paiml/pforge"
documentation = "https://docs.rs/pforge-runtime"
```

### Individual Crate Configuration

Each crate inherits with `.workspace = true`:

```toml
[package]
name = "pforge-config"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true
```

**Benefits**:
- Update version once, applies to all crates
- Consistent metadata across workspace
- Less duplication
- Easier maintenance

**Note**: Individual crates can override workspace values if needed. For example, `pforge-cli` might have a different description than the workspace default.

## Choosing Keywords and Categories

### Keywords

crates.io allows up to **5 keywords**, each max **20 characters**. Choose carefully for discoverability.

**pforge's keyword strategy**:

```toml
keywords = ["mcp", "codegen", "tdd", "framework", "declarative"]
```

We chose:
- **mcp**: Primary domain (Model Context Protocol)
- **codegen**: Key feature (code generation)
- **tdd**: Methodology (test-driven development)
- **framework**: What it is
- **declarative**: How it works

**Avoid**:
- Generic terms ("rust", "server") - too broad
- Duplicate concepts ("framework" + "library")
- Marketing terms ("fast", "best")
- Longer than 20 chars (will be rejected)

**Test keyword effectiveness**:

Search crates.io for each keyword to see competition and relevance.

### Categories

Categories come from a predefined list: https://crates.io/categories

**pforge's categories**:

```toml
categories = ["development-tools", "web-programming", "command-line-utilities"]
```

Reasoning:
- **development-tools**: Primary category (tool for developers)
- **web-programming**: MCP is web/network protocol
- **command-line-utilities**: pforge is a CLI tool

**Available categories include**:
- algorithms
- api-bindings
- asynchronous
- authentication
- caching
- command-line-utilities
- config
- cryptography
- database
- development-tools
- encoding
- parsing
- web-programming

Choose 2-3 most relevant categories. Don't over-categorize.

## License Selection

The `license` field uses SPDX identifiers: https://spdx.org/licenses/

**Common choices**:

- **MIT**: Permissive, simple, widely used
- **Apache-2.0**: Permissive, patent grant, corporate-friendly
- **MIT OR Apache-2.0**: Dual license (common in Rust ecosystem)
- **BSD-3-Clause**: Permissive, attribution required
- **GPL-3.0**: Copyleft, viral license

**pforge uses MIT**:

```toml
license = "MIT"
```

Simple, permissive, minimal restrictions. Good for libraries and frameworks where you want maximum adoption.

**For dual licensing**:

```toml
license = "MIT OR Apache-2.0"
```

**For custom licenses**:

```toml
license-file = "LICENSE.txt"
```

Points to a custom license file (rare, not recommended).

**Include license file**: Always add `LICENSE` or `LICENSE-MIT` file to repository root, even when using SPDX identifier.

## Including Files in the Package

By default, cargo includes all source files but excludes:
- `.git/`
- `target/`
- Files in `.gitignore`

### The `include` Field

For crates needing specific files (like templates), use `include`:

```toml
[package]
name = "pforge-cli"
# ... other fields ...
include = [
    "src/**/*",
    "templates/**/*",
    "Cargo.toml",
    "README.md",
    "LICENSE",
]
```

**When pforge-cli was first published without `include`**:

```bash
$ cargo install pforge-cli
$ pforge new my-project
Error: template directory not found
```

The `templates/` directory wasn't included! Adding `include` fixed it.

### The `exclude` Field

Alternatively, exclude specific files:

```toml
exclude = [
    "tests/fixtures/large_file.bin",
    "benches/data/*",
    ".github/",
]
```

Use `include` (allowlist) or `exclude` (blocklist), not both.

### Verify Package Contents

Before publishing, check what will be included:

```bash
cargo package --list
```

Example output:

```
pforge-cli-0.1.0/Cargo.toml
pforge-cli-0.1.0/src/main.rs
pforge-cli-0.1.0/src/commands/mod.rs
pforge-cli-0.1.0/src/commands/new.rs
pforge-cli-0.1.0/templates/new-project/pforge.yaml.template
pforge-cli-0.1.0/templates/new-project/Cargo.toml.template
pforge-cli-0.1.0/README.md
pforge-cli-0.1.0/LICENSE
```

Review this list carefully. Missing files cause runtime errors. Extra files increase download size.

### Inspect the Package

Create the package without publishing:

```bash
cargo package
```

This creates `target/package/pforge-cli-0.1.0.crate`. Inspect it:

```bash
tar -tzf target/package/pforge-cli-0.1.0.crate | head -20
```

Extract and examine:

```bash
cd target/package
tar -xzf pforge-cli-0.1.0.crate
cd pforge-cli-0.1.0
tree
```

This lets you verify the exact contents users will download.

## Writing the README

The README is the first thing users see on crates.io and docs.rs. Make it count.

### Essential README Sections

**pforge-config's README structure**:

```markdown
# pforge-config

Configuration parsing and validation for pforge MCP servers.

## Overview

pforge-config provides the core configuration types used by the pforge
framework. It parses YAML configurations and validates them against
the MCP server schema.

## Installation

Add to your `Cargo.toml`:

[dependencies]
pforge-config = "0.1.0"

## Quick Example

\`\`\`rust
use pforge_config::ForgeConfig;

let yaml = r#"
forge:
  name: my-server
  version: 0.1.0
tools:
  - name: greet
    type: native
"#;

let config = ForgeConfig::from_yaml(yaml)?;
println!("Server: {}", config.name);
\`\`\`

## Features

- YAML configuration parsing
- Schema validation
- Type-safe configuration structs
- Comprehensive error messages

## Documentation

Full documentation available at https://docs.rs/pforge-config

## License

MIT
```

### README Best Practices

1. **Start with one-line description**: Same as `Cargo.toml` description
2. **Show installation**: Copy-paste `Cargo.toml` snippet
3. **Provide quick example**: Working code in first 20 lines
4. **Highlight features**: Bullet points, not paragraphs
5. **Link to docs**: Don't duplicate full API docs in README
6. **Keep it short**: 100-200 lines max
7. **Use badges** (optional): Build status, crates.io version, docs.rs

### Badges Example

```markdown
[![Crates.io](https://img.shields.io/crates/v/pforge-config.svg)](https://crates.io/crates/pforge-config)
[![Documentation](https://docs.rs/pforge-config/badge.svg)](https://docs.rs/pforge-config)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
```

Badges provide quick status at a glance.

## Version Specifications for Dependencies

### External Dependencies

For dependencies from crates.io, use **caret requirements** (default):

```toml
[dependencies]
serde = "1.0"          # Means >=1.0.0, <2.0.0
serde_json = "1.0.108" # Means >=1.0.108, <2.0.0
thiserror = "1.0"
```

This allows minor and patch updates automatically (following semver).

**Alternative version syntax**:

```toml
serde = "^1.0"      # Explicit caret (same as "1.0")
serde = "~1.0.100"  # Tilde: >=1.0.100, <1.1.0
serde = ">=1.0"     # Unbounded (not recommended)
serde = "=1.0.100"  # Exact version (too strict)
```

**Recommendation**: Use simple version like `"1.0"` for libraries, `"=1.0.100"` only for binaries if needed.

### Internal Dependencies (Workspace)

For crates within the same workspace, use **workspace dependencies**:

```toml
[workspace.dependencies]
pforge-config = { path = "crates/pforge-config", version = "0.1.0" }
pforge-macro = { path = "crates/pforge-macro", version = "0.1.0" }
pforge-runtime = { path = "crates/pforge-runtime", version = "0.1.0" }
```

Each crate references with:

```toml
[dependencies]
pforge-config = { workspace = true }
```

**Critical**: Both `path` and `version` are required. The `path` is used for local development. The `version` is used when published to crates.io.

### What Happens Without Version

If you forget `version` on internal dependencies:

```toml
# WRONG - will fail to publish
pforge-config = { path = "../pforge-config" }
```

Publishing fails:

```
error: all dependencies must specify a version for published crates
  --> Cargo.toml:15:1
   |
15 | pforge-config = { path = "../pforge-config" }
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Fix**: Add explicit version:

```toml
# CORRECT
pforge-config = { path = "../pforge-config", version = "0.1.0" }
```

Or use workspace inheritance:

```toml
# In workspace root Cargo.toml
[workspace.dependencies]
pforge-config = { path = "crates/pforge-config", version = "0.1.0" }

# In dependent crate
[dependencies]
pforge-config = { workspace = true }
```

### Optional Dependencies

For features that are optional:

```toml
[dependencies]
serde = { version = "1.0", optional = true }

[features]
default = []
serialization = ["serde"]
```

Users can enable with:

```toml
pforge-config = { version = "0.1.0", features = ["serialization"] }
```

## Preparing Each pforge Crate

Here's how we prepared each crate:

### pforge-config (Foundation Crate)

**Cargo.toml**:

```toml
[package]
name = "pforge-config"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
serde_yml = { workspace = true }
thiserror = { workspace = true }
url = "2.5"
```

**No special includes needed** - all source files in `src/` are automatically included.

**README**: 150 lines, installation + quick example + features

### pforge-macro (Procedural Macro Crate)

**Cargo.toml**:

```toml
[package]
name = "pforge-macro"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

**Key**: `proc-macro = true` required for procedural macro crates.

**No dependencies on other pforge crates** - macros are independent.

### pforge-runtime (Depends on Config)

**Cargo.toml**:

```toml
[package]
name = "pforge-runtime"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }

# Internal dependency - requires pforge-config published first
pforge-config = { workspace = true }

# Runtime-specific
pmcp = "1.6"
schemars = { version = "0.8", features = ["derive"] }
rustc-hash = "2.0"
dashmap = "6.0"
reqwest = { version = "0.12", features = ["json"] }
```

**Critical**: `pforge-config` must be published to crates.io before `pforge-runtime` can be published.

### pforge-codegen (Depends on Config)

**Cargo.toml**:

```toml
[package]
name = "pforge-codegen"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

# Internal dependency
pforge-config = { workspace = true }

# Codegen-specific
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

Can be published in parallel with `pforge-runtime` since both only depend on `pforge-config`.

### pforge-cli (Depends on Everything)

**Cargo.toml**:

```toml
[package]
name = "pforge-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description.workspace = true
keywords.workspace = true
categories.workspace = true
homepage.workspace = true
documentation.workspace = true

# CRITICAL: Include templates directory
include = [
    "src/**/*",
    "templates/**/*",
    "Cargo.toml",
    "README.md",
]

[[bin]]
name = "pforge"
path = "src/main.rs"

[dependencies]
# All internal dependencies must be published first
pforge-runtime = { workspace = true }
pforge-config = { workspace = true }
pforge-codegen = { workspace = true }

# CLI-specific
anyhow = { workspace = true }
clap = { version = "4.4", features = ["derive"] }
tokio = { workspace = true }
```

**Must be published last** - depends on all other pforge crates.

**Critical**: The `include` field ensures templates are bundled.

## Pre-Publication Checklist Per Crate

Before publishing each crate, verify:

### Metadata Checklist

- [ ] `name` is unique on crates.io
- [ ] `version` follows semver
- [ ] `edition` is set (2021 recommended)
- [ ] `license` uses SPDX identifier
- [ ] `description` is clear and concise
- [ ] `repository` links to source code
- [ ] `keywords` are relevant (max 5, each max 20 chars)
- [ ] `categories` are from official list
- [ ] `authors` are credited
- [ ] `readme` path is correct

### Files Checklist

- [ ] `README.md` exists and is comprehensive
- [ ] `LICENSE` file exists
- [ ] Required files are included (check with `cargo package --list`)
- [ ] Templates/resources are in `include` if needed
- [ ] No unnecessary files (large test data, etc.)
- [ ] Package size is reasonable (<5MB for libraries)

### Dependencies Checklist

- [ ] All internal dependencies have `version` specified
- [ ] Internal dependencies are published to crates.io
- [ ] External dependency versions are appropriate
- [ ] No `path` dependencies without `version`
- [ ] Optional dependencies have corresponding features

### Code Checklist

- [ ] All tests pass: `cargo test`
- [ ] Clippy is clean: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] No `TODO` or `FIXME` in public APIs
- [ ] Public APIs have doc comments

### Testing Checklist

- [ ] Dry run succeeds: `cargo publish --dry-run`
- [ ] Package contents verified: `cargo package --list`
- [ ] Package size is acceptable: check `target/package/*.crate`
- [ ] README renders correctly on GitHub
- [ ] Examples compile and run

## Common Preparation Mistakes

### 1. Missing README

**Problem**: No `README.md` file.

**Error**:

```
warning: manifest has no readme or documentation
```

Not fatal, but strongly discouraged. Users won't know how to use your crate.

**Fix**: Write a README with installation and examples.

### 2. Keywords Too Long

**Problem**: Keywords exceed 20 characters.

**Error**:

```
error: keyword "model-context-protocol" is too long (max 20 chars)
```

**Fix**: Abbreviate or rephrase. Use "mcp" instead of "model-context-protocol".

### 3. Invalid Category

**Problem**: Category not in official list.

**Error**:

```
error: category "mcp-servers" is not a valid crates.io category
```

**Fix**: Choose from https://crates.io/categories. Use "web-programming" or "development-tools".

### 4. Huge Package Size

**Problem**: Accidentally including large test data files.

**Warning**:

```
warning: package size is 45.2 MB
note: crates.io has a 10MB package size limit
```

**Fix**: Use `exclude` or `include` to remove large files. Move test data to separate repository.

### 5. Broken Links in README

**Problem**: README links use relative paths that don't work on crates.io.

**Example**:

```markdown
```

This breaks on crates.io because `docs/` isn't included.

**Fix**: Use absolute URLs:

```markdown
```

Or include the file:

```toml
include = ["docs/architecture.png"]
```

## Automation Scripts

Create a script to prepare all crates:

```bash
#!/bin/bash
# scripts/prepare-publish.sh

set -e

echo "Preparing crates for publication..."

# Check all tests pass
echo "Running tests..."
cargo test --all

# Check formatting
echo "Checking formatting..."
cargo fmt --check

# Check clippy
echo "Running clippy..."
cargo clippy --all -- -D warnings

# Build documentation
echo "Building docs..."
cargo doc --all --no-deps

# Dry run for each publishable crate
for crate in pforge-config pforge-macro pforge-runtime pforge-codegen pforge-cli; do
    echo "Dry run: $crate"
    cd "crates/$crate"
    cargo publish --dry-run
    cargo package --list > /tmp/${crate}-files.txt
    echo "  Files: $(wc -l < /tmp/${crate}-files.txt)"
    cd ../..
done

echo "All crates ready for publication!"
```

Run before publishing:

```bash
./scripts/prepare-publish.sh
```

## Summary

Preparing crates for publication requires:

1. **Complete metadata**: description, license, keywords, categories
2. **Workspace inheritance**: Share common metadata across crates
3. **Correct file inclusion**: Use `include` for templates/resources
4. **Version specifications**: Internal dependencies need `version` + `path`
5. **Comprehensive README**: Installation, examples, features
6. **Verification**: Test dry runs, inspect packages, review file lists

pforge's preparation process caught multiple issues:
- Missing templates in CLI crate
- Keywords exceeding 20 characters
- Missing version on internal dependencies
- Broken documentation links

Running thorough checks before publication saves time and prevents bad releases.

---

**Next**: [Version Management](ch17-02-versioning.md)
