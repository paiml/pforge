#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("REST API Proxy Example v0.1.0");
    println!();
    println!("This example demonstrates:");
    println!("- HTTP handler for REST APIs");
    println!("- Template-based endpoints");
    println!("- Custom headers");
    println!();
    println!("Available tools:");
    println!("  get_user(username) - Get GitHub user info");
    println!("  get_repos(username) - Get user repositories");
    println!("  search_repos(query) - Search repositories");
    println!();
    println!("See pforge.yaml for configuration!");

    Ok(())
}
