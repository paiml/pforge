use anyhow::Result;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use pforge_config::parse_config;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Run the server in development mode with optional hot reload
pub async fn execute(config_path: &str, watch: bool) -> Result<()> {
    println!("Starting pforge in development mode...");
    println!("  Config: {}", config_path);
    println!("  Watch: {}", watch);

    if !watch {
        // No hot reload - just run serve mode
        return super::serve::execute(config_path).await;
    }

    // Hot reload enabled
    println!("\n🔄 Hot reload enabled - watching for changes...");

    let should_reload = Arc::new(AtomicBool::new(false));
    let should_reload_watcher = should_reload.clone();

    // Create channel for file change notifications
    let (tx, mut rx) = mpsc::channel::<()>(1);

    // Set up file watcher
    let config_path_owned = config_path.to_string();
    let watcher_tx = tx.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let mut debouncer = new_debouncer(
                Duration::from_millis(500),
                move |result: DebounceEventResult| {
                    if let Ok(events) = result {
                        if !events.is_empty() {
                            should_reload_watcher.store(true, Ordering::SeqCst);
                            let _ = watcher_tx.try_send(());
                        }
                    }
                },
            )
            .expect("Failed to create file watcher");

            // Watch the config file's parent directory
            let config_file = Path::new(&config_path_owned);
            let watch_dir = config_file.parent().unwrap_or(Path::new("."));

            debouncer
                .watcher()
                .watch(watch_dir, RecursiveMode::Recursive)
                .expect("Failed to watch directory");

            println!("  Watching: {}", watch_dir.display());

            // Keep the watcher alive
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    });

    // Main server loop with hot reload
    loop {
        println!("\n📦 Loading configuration...");

        // Load and validate config
        let config = match parse_config(Path::new(config_path)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Configuration error: {}", e);
                eprintln!("   Waiting for changes...");

                // Wait for file change to retry
                rx.recv().await;
                should_reload.store(false, Ordering::SeqCst);
                continue;
            }
        };

        println!("✅ Loaded: {} v{}", config.forge.name, config.forge.version);
        println!("   Tools: {}", config.tools.len());

        // Create and run server
        let server = pforge_runtime::McpServer::new(config);

        // Run server until reload is triggered
        tokio::select! {
            result = server.run() => {
                match result {
                    Ok(()) => {
                        println!("Server stopped normally");
                        break;
                    }
                    Err(e) => {
                        eprintln!("❌ Server error: {}", e);
                        eprintln!("   Waiting for changes...");
                    }
                }
            }
            _ = rx.recv() => {
                if should_reload.load(Ordering::SeqCst) {
                    println!("\n🔄 Changes detected - reloading...");
                    should_reload.store(false, Ordering::SeqCst);
                    // Server will be restarted in next loop iteration
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_dev_execute_invalid_config() {
        // Test with non-existent config file
        let result = execute("/nonexistent/config.yaml", false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dev_watch_mode_setup() {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio
  optimization: debug

tools: []
"#
        )
        .unwrap();

        // Test that watch mode can be set up (will fail on actual run due to stdio)
        // This just validates the config loading path works
        let config_path = temp_file.path();
        let config = parse_config(config_path);
        assert!(config.is_ok());
    }
}
