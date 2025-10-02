use anyhow::Result;
use std::process::Command;

pub fn execute(release: bool) -> Result<()> {
    println!("Building pforge server...");

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
        println!("  Mode: release");
    } else {
        println!("  Mode: debug");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("✓ Build successful!");
    } else {
        anyhow::bail!("Build failed");
    }

    Ok(())
}
