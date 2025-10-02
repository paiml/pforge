use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const CARGO_TOML_TEMPLATE: &str = include_str!("../../templates/new-project/Cargo.toml.template");
const PFORGE_YAML_TEMPLATE: &str = include_str!("../../templates/new-project/pforge.yaml.template");
const MAIN_RS_TEMPLATE: &str = include_str!("../../templates/new-project/src/main.rs.template");
const HANDLERS_MOD_TEMPLATE: &str =
    include_str!("../../templates/new-project/src/handlers/mod.rs.template");
const HELLO_RS_TEMPLATE: &str =
    include_str!("../../templates/new-project/src/handlers/hello.rs.template");

pub fn execute(name: &str, path: Option<&str>) -> Result<()> {
    let target_dir = if let Some(p) = path {
        Path::new(p).join(name)
    } else {
        Path::new(name).to_path_buf()
    };

    println!("Creating new pforge project: {}", name);
    println!("  Location: {}", target_dir.display());

    // Create directory structure
    fs::create_dir_all(&target_dir).context("Failed to create project directory")?;

    fs::create_dir_all(target_dir.join("src/handlers"))
        .context("Failed to create src/handlers directory")?;

    // Write files with variable substitution
    let cargo_toml = CARGO_TOML_TEMPLATE.replace("{{PROJECT_NAME}}", name);
    fs::write(target_dir.join("Cargo.toml"), cargo_toml).context("Failed to write Cargo.toml")?;

    let pforge_yaml = PFORGE_YAML_TEMPLATE.replace("{{PROJECT_NAME}}", name);
    fs::write(target_dir.join("pforge.yaml"), pforge_yaml)
        .context("Failed to write pforge.yaml")?;

    let main_rs = MAIN_RS_TEMPLATE.replace("{{PROJECT_NAME}}", name);
    fs::write(target_dir.join("src/main.rs"), main_rs).context("Failed to write src/main.rs")?;

    fs::write(
        target_dir.join("src/handlers/mod.rs"),
        HANDLERS_MOD_TEMPLATE,
    )
    .context("Failed to write src/handlers/mod.rs")?;

    fs::write(target_dir.join("src/handlers/hello.rs"), HELLO_RS_TEMPLATE)
        .context("Failed to write src/handlers/hello.rs")?;

    println!("✓ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  pforge serve");

    Ok(())
}
