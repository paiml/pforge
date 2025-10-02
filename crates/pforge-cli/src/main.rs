mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pforge")]
#[command(about = "Declarative MCP server framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new pforge project
    New {
        /// Project name
        name: String,

        /// Target directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Build the pforge server
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Run the pforge server
    Serve {
        /// Path to pforge.yaml config
        #[arg(short, long, default_value = "pforge.yaml")]
        config: String,
    },

    /// Development mode with hot reload
    Dev {
        /// Path to pforge.yaml config
        #[arg(short, long, default_value = "pforge.yaml")]
        config: String,

        /// Watch for changes
        #[arg(short, long, default_value_t = true)]
        watch: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, path } => {
            commands::new::execute(&name, path.as_deref())?;
        }
        Commands::Build { release } => {
            commands::build::execute(release)?;
        }
        Commands::Serve { config } => {
            commands::serve::execute(&config).await?;
        }
        Commands::Dev { config, watch } => {
            commands::dev::execute(&config, watch).await?;
        }
    }

    Ok(())
}
