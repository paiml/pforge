use anyhow::Result;

pub async fn execute(config_path: &str, watch: bool) -> Result<()> {
    println!("Starting pforge in development mode...");
    println!("  Config: {}", config_path);
    println!("  Watch: {}", watch);

    // TODO: Hot reload implementation
    println!("\n⚠ Development mode with hot reload pending");
    println!("  Falling back to serve mode...");

    super::serve::execute(config_path).await
}
