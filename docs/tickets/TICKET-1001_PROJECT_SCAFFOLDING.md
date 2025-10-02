# TICKET-1001: Project Scaffolding and Build System

**Phase**: 1 - Foundation
**Cycle**: 1
**Priority**: Critical
**Estimated Time**: 2 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD

---

## Objective

Create the foundational Cargo workspace structure, build system, and project scaffolding infrastructure for pforge. This includes setting up 5 crates with proper dependencies, PMAT quality gate integration, and the ability to generate new pforge projects from templates.

---

## Problem Statement

Currently, pforge is an empty repository. We need to:
1. Establish a Cargo workspace with 5 crates (cli, runtime, codegen, config, macro)
2. Configure proper inter-crate dependencies
3. Integrate PMAT quality gates (.pmat/ directory, pre-commit hooks)
4. Create project templates for `pforge new <name>` command scaffolding

This ticket lays the foundation for all subsequent development.

---

## Technical Requirements

### Must Implement

1. **Cargo Workspace**: Root `Cargo.toml` with workspace members
2. **Five Crates**:
   - `pforge-cli`: Binary crate for CLI commands
   - `pforge-runtime`: Library for handler registry, transport, state
   - `pforge-codegen`: Library for code generation from YAML
   - `pforge-config`: Library for configuration parsing
   - `pforge-macro`: Proc-macro crate for derive macros
3. **PMAT Integration**:
   - `.pmat/quality-gates.yaml` configuration
   - Pre-commit hook script
   - Quality gate runner
4. **Project Templates**:
   - `templates/new-project/` directory
   - Template files for `pforge.yaml`, `src/main.rs`, etc.

### Dependencies to Add

```toml
# Common dependencies across crates
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yml = "0.0.10"
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# pforge-runtime specific
pmcp = "1.6"  # MCP SDK
schemars = { version = "0.8", features = ["derive"] }
dashmap = "6.0"
rustc-hash = "2.0"  # FxHashMap

# pforge-codegen specific
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"

# pforge-cli specific
clap = { version = "4.4", features = ["derive"] }
```

---

## API Design

### Workspace Structure

```
pforge/
├── Cargo.toml                    # Workspace root
├── .pmat/
│   └── quality-gates.yaml        # PMAT configuration
├── scripts/
│   └── pre-commit.sh            # Git hook script
├── templates/
│   └── new-project/
│       ├── Cargo.toml.template
│       ├── pforge.yaml.template
│       └── src/
│           └── main.rs.template
├── crates/
│   ├── pforge-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │           └── mod.rs
│   ├── pforge-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── pforge-codegen/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── pforge-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── pforge-macro/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── tests/
    └── scaffold_tests.rs
```

### .pmat/quality-gates.yaml

```yaml
gates:
  - name: complexity
    max_cyclomatic: 20
    max_cognitive: 15
    fail_on_violation: true

  - name: satd
    max_count: 0
    fail_on_violation: true

  - name: test_coverage
    min_line_coverage: 80
    min_branch_coverage: 75
    fail_on_violation: true

  - name: tdg_score
    min_grade: 0.75
    fail_on_violation: true
```

---

## EXTREME TDD: RED Phase Tests

All tests must be written FIRST and must FAIL initially.

### Test File: `tests/scaffold_tests.rs`

```rust
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn red_test_workspace_compiles() {
    // Expected: ❌ FAIL - Workspace doesn't exist yet
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .output()
        .expect("Failed to run cargo build");

    assert!(output.status.success(), "Workspace should compile");
}

#[test]
fn red_test_all_crates_exist() {
    // Expected: ❌ FAIL - Crates don't exist yet
    let crates = vec![
        "crates/pforge-cli",
        "crates/pforge-runtime",
        "crates/pforge-codegen",
        "crates/pforge-config",
        "crates/pforge-macro",
    ];

    for crate_path in crates {
        let path = PathBuf::from(crate_path);
        assert!(path.exists(), "Crate {} should exist", crate_path);

        let cargo_toml = path.join("Cargo.toml");
        assert!(cargo_toml.exists(), "Cargo.toml should exist in {}", crate_path);
    }
}

#[test]
fn red_test_pmat_configuration_exists() {
    // Expected: ❌ FAIL - PMAT config doesn't exist
    let pmat_config = PathBuf::from(".pmat/quality-gates.yaml");
    assert!(pmat_config.exists(), "PMAT quality gates config should exist");
}

#[test]
fn red_test_workspace_has_correct_members() {
    // Expected: ❌ FAIL - Workspace Cargo.toml doesn't have members
    let cargo_toml = std::fs::read_to_string("Cargo.toml")
        .expect("Should read root Cargo.toml");

    assert!(cargo_toml.contains("workspace"));
    assert!(cargo_toml.contains("pforge-cli"));
    assert!(cargo_toml.contains("pforge-runtime"));
    assert!(cargo_toml.contains("pforge-codegen"));
    assert!(cargo_toml.contains("pforge-config"));
    assert!(cargo_toml.contains("pforge-macro"));
}

#[test]
fn red_test_runtime_crate_has_pmcp_dependency() {
    // Expected: ❌ FAIL - Runtime Cargo.toml doesn't exist
    let cargo_toml = std::fs::read_to_string("crates/pforge-runtime/Cargo.toml")
        .expect("Should read runtime Cargo.toml");

    assert!(cargo_toml.contains("pmcp"), "Runtime should depend on pmcp");
    assert!(cargo_toml.contains("tokio"), "Runtime should depend on tokio");
    assert!(cargo_toml.contains("async-trait"), "Runtime should depend on async-trait");
}

#[test]
fn red_test_cli_crate_has_clap_dependency() {
    // Expected: ❌ FAIL - CLI Cargo.toml doesn't exist
    let cargo_toml = std::fs::read_to_string("crates/pforge-cli/Cargo.toml")
        .expect("Should read CLI Cargo.toml");

    assert!(cargo_toml.contains("clap"), "CLI should depend on clap");
}

#[test]
fn red_test_codegen_crate_has_syn_quote() {
    // Expected: ❌ FAIL - Codegen Cargo.toml doesn't exist
    let cargo_toml = std::fs::read_to_string("crates/pforge-codegen/Cargo.toml")
        .expect("Should read codegen Cargo.toml");

    assert!(cargo_toml.contains("syn"), "Codegen should depend on syn");
    assert!(cargo_toml.contains("quote"), "Codegen should depend on quote");
}

#[test]
fn red_test_project_templates_exist() {
    // Expected: ❌ FAIL - Templates don't exist
    let templates = vec![
        "templates/new-project/Cargo.toml.template",
        "templates/new-project/pforge.yaml.template",
        "templates/new-project/src/main.rs.template",
    ];

    for template in templates {
        let path = PathBuf::from(template);
        assert!(path.exists(), "Template {} should exist", template);
    }
}

#[test]
fn red_test_pre_commit_hook_exists() {
    // Expected: ❌ FAIL - Pre-commit hook doesn't exist
    let hook = PathBuf::from("scripts/pre-commit.sh");
    assert!(hook.exists(), "Pre-commit hook should exist");

    let content = std::fs::read_to_string(&hook)
        .expect("Should read pre-commit hook");

    assert!(content.contains("pmat"), "Pre-commit should run PMAT checks");
}

#[test]
fn red_test_all_crates_compile_independently() {
    // Expected: ❌ FAIL - Crates don't exist or don't compile
    let crates = vec![
        "pforge-runtime",
        "pforge-config",
        "pforge-codegen",
        "pforge-macro",
        "pforge-cli",
    ];

    for crate_name in crates {
        let output = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg(crate_name)
            .output()
            .expect("Failed to run cargo build");

        assert!(
            output.status.success(),
            "Crate {} should compile independently",
            crate_name
        );
    }
}
```

---

## GREEN Phase: Minimal Implementation

### Step 1: Create Workspace Root

**File**: `Cargo.toml`

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

[workspace.dependencies]
# Common dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yml = "0.0.10"
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Internal crates
pforge-runtime = { path = "crates/pforge-runtime" }
pforge-config = { path = "crates/pforge-config" }
pforge-codegen = { path = "crates/pforge-codegen" }
pforge-macro = { path = "crates/pforge-macro" }
```

### Step 2: Create Each Crate

**Files to Create**:

1. `crates/pforge-cli/Cargo.toml`
2. `crates/pforge-cli/src/main.rs`
3. `crates/pforge-runtime/Cargo.toml`
4. `crates/pforge-runtime/src/lib.rs`
5. `crates/pforge-config/Cargo.toml`
6. `crates/pforge-config/src/lib.rs`
7. `crates/pforge-codegen/Cargo.toml`
8. `crates/pforge-codegen/src/lib.rs`
9. `crates/pforge-macro/Cargo.toml`
10. `crates/pforge-macro/src/lib.rs`

### Step 3: Create PMAT Configuration

**File**: `.pmat/quality-gates.yaml`

```yaml
gates:
  - name: complexity
    max_cyclomatic: 20
    max_cognitive: 15
    fail_on_violation: true

  - name: satd
    max_count: 0
    fail_on_violation: true

  - name: test_coverage
    min_line_coverage: 80
    min_branch_coverage: 75
    fail_on_violation: true

  - name: tdg_score
    min_grade: 0.75
    fail_on_violation: true
```

### Step 4: Create Pre-Commit Hook

**File**: `scripts/pre-commit.sh`

```bash
#!/bin/bash
set -e

echo "Running pforge quality gates..."

# Format check
cargo fmt --check

# Clippy
cargo clippy --all-targets -- -D warnings

# Tests
cargo test --all

# PMAT quality gates (if pmat is installed)
if command -v pmat &> /dev/null; then
    pmat analyze complexity --max 20
    pmat analyze satd --max 0
    pmat analyze tdg --min 0.75
fi

echo "✓ All quality gates passed"
```

### Step 5: Create Project Templates

**File**: `templates/new-project/Cargo.toml.template`
**File**: `templates/new-project/pforge.yaml.template`
**File**: `templates/new-project/src/main.rs.template`

---

## REFACTOR Phase

After tests pass (GREEN), refactor with quality gates:

1. Run `cargo fmt`
2. Run `cargo clippy -- -D warnings`
3. Run `pmat analyze complexity --max 20`
4. Run `pmat analyze tdg --min 0.75`
5. Ensure all tests still pass

---

## Acceptance Criteria

- [x] `cargo build --release` succeeds for entire workspace
- [x] All 5 crates compile independently
- [x] PMAT quality gates configuration exists
- [x] Pre-commit hook script exists and is executable
- [x] Project templates exist
- [x] All 10 RED tests now PASS (GREEN)
- [x] Code passes clippy with zero warnings
- [x] Code formatted with `cargo fmt`
- [x] Quality metrics: complexity <20, TDG >0.75, SATD=0

---

## Time Tracking

- **RED Phase**: 30 minutes (write tests)
- **GREEN Phase**: 60 minutes (minimal implementation)
- **REFACTOR Phase**: 15 minutes (clean up, quality gates)
- **Documentation**: 15 minutes
- **Total**: 2 hours

---

## Next Ticket

**TICKET-1002**: YAML Configuration Schema and Parser

---

## Notes

- This ticket establishes the foundation. All subsequent tickets depend on this structure.
- PMAT integration is critical - install `pmat` if not available: `cargo install pmat`
- Pre-commit hook should be installed: `ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit`

---

**Status**: 📋 Ready for Development
**Created**: 2025-10-02
**Assignee**: Developer implementing pforge spec
