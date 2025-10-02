// TICKET-1001: Project Scaffolding Tests (RED Phase)
// These tests MUST FAIL initially

use std::path::PathBuf;
use std::process::Command;

// Helper to get workspace root
fn workspace_root() -> PathBuf {
    let output = Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .expect("Failed to locate workspace");

    let cargo_toml_path = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .trim()
        .to_string();

    PathBuf::from(cargo_toml_path)
        .parent()
        .expect("Cargo.toml should have parent")
        .to_path_buf()
}

#[test]
fn red_test_workspace_compiles() {
    // Expected: ✅ PASS - Workspace already compiles
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .output()
        .expect("Failed to run cargo build");

    assert!(output.status.success(), "Workspace should compile");
}

#[test]
fn red_test_all_crates_exist() {
    // Expected: ✅ PASS - Crates already exist
    let root = workspace_root();
    let crates = vec![
        "crates/pforge-cli",
        "crates/pforge-runtime",
        "crates/pforge-codegen",
        "crates/pforge-config",
        "crates/pforge-macro",
    ];

    for crate_path in crates {
        let path = root.join(crate_path);
        assert!(path.exists(), "Crate {} should exist", crate_path);

        let cargo_toml = path.join("Cargo.toml");
        assert!(
            cargo_toml.exists(),
            "Cargo.toml should exist in {}",
            crate_path
        );
    }
}

#[test]
fn red_test_pmat_configuration_exists() {
    // Expected: ✅ PASS - Already created
    let root = workspace_root();
    let pmat_config = root.join(".pmat/quality-gates.yaml");
    assert!(
        pmat_config.exists(),
        "PMAT quality gates config should exist"
    );
}

#[test]
fn red_test_workspace_has_correct_members() {
    // Expected: ✅ PASS - Workspace already configured
    let root = workspace_root();
    let cargo_toml =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("Should read root Cargo.toml");

    assert!(cargo_toml.contains("workspace"));
    assert!(cargo_toml.contains("pforge-cli"));
    assert!(cargo_toml.contains("pforge-runtime"));
    assert!(cargo_toml.contains("pforge-codegen"));
    assert!(cargo_toml.contains("pforge-config"));
    assert!(cargo_toml.contains("pforge-macro"));
}

#[test]
fn red_test_runtime_crate_has_pmcp_dependency() {
    // Expected: ✅ PASS - Already configured
    let root = workspace_root();
    let cargo_toml = std::fs::read_to_string(root.join("crates/pforge-runtime/Cargo.toml"))
        .expect("Should read runtime Cargo.toml");

    assert!(cargo_toml.contains("pmcp"), "Runtime should depend on pmcp");
    assert!(
        cargo_toml.contains("tokio"),
        "Runtime should depend on tokio"
    );
    assert!(
        cargo_toml.contains("async-trait"),
        "Runtime should depend on async-trait"
    );
}

#[test]
fn red_test_cli_crate_has_clap_dependency() {
    // Expected: ✅ PASS - Already configured
    let root = workspace_root();
    let cargo_toml = std::fs::read_to_string(root.join("crates/pforge-cli/Cargo.toml"))
        .expect("Should read CLI Cargo.toml");

    assert!(cargo_toml.contains("clap"), "CLI should depend on clap");
}

#[test]
fn red_test_codegen_crate_has_syn_quote() {
    // Expected: ✅ PASS - Already configured
    let root = workspace_root();
    let cargo_toml = std::fs::read_to_string(root.join("crates/pforge-codegen/Cargo.toml"))
        .expect("Should read codegen Cargo.toml");

    assert!(cargo_toml.contains("syn"), "Codegen should depend on syn");
    assert!(
        cargo_toml.contains("quote"),
        "Codegen should depend on quote"
    );
}

#[test]
fn red_test_project_templates_exist() {
    // Expected: ✅ PASS - Templates exist
    let root = workspace_root();
    let templates = vec![
        "templates/new-project/Cargo.toml.template",
        "templates/new-project/pforge.yaml.template",
        "templates/new-project/src/main.rs.template",
    ];

    for template in templates {
        let path = root.join(template);
        assert!(path.exists(), "Template {} should exist", template);
    }
}

#[test]
fn red_test_pre_commit_hook_exists() {
    // Expected: ✅ PASS - Already created
    let root = workspace_root();
    let hook = root.join("scripts/pre-commit.sh");
    assert!(hook.exists(), "Pre-commit hook should exist");

    let content = std::fs::read_to_string(&hook).expect("Should read pre-commit hook");

    assert!(
        content.contains("pmat") || content.contains("PMAT") || content.contains("cargo"),
        "Pre-commit should reference checks"
    );
}

#[test]
fn red_test_all_crates_compile_independently() {
    // Expected: ✅ PASS - Already compiling
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
            "Crate {} should compile independently. stderr: {}",
            crate_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
