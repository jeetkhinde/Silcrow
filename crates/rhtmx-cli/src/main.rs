mod commands;
mod project;
mod theme;

#[cfg(feature = "dev-server")]
mod dev;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "rhtmx")]
#[command(version, about = "RHTMX CLI - Rust + HTMX Framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new RHTMX project
    New {
        /// Project name
        name: String,

        /// Theme to use (optional)
        #[arg(short, long)]
        theme: Option<String>,

        /// Skip git initialization
        #[arg(long)]
        no_git: bool,
    },

    /// Start development server with hot reload
    Dev {
        /// Port to run the server on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Build project for deployment
    Build {
        /// Build mode: ssr (Server-Side Rendering), ssg (Static Site Generation), or isr (Incremental Static Regeneration)
        #[arg(short, long, default_value = "ssr")]
        mode: BuildMode,

        /// Release build (optimized)
        #[arg(short, long)]
        release: bool,
    },

    /// Theme management commands
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum BuildMode {
    /// Server-Side Rendering (default)
    Ssr,
    /// Static Site Generation
    Ssg,
    /// Incremental Static Regeneration
    Isr,
}

#[derive(Subcommand)]
enum ThemeCommands {
    /// Initialize a new theme
    Init {
        /// Theme name
        name: String,
    },

    /// Clear theme cache
    #[command(name = "cache-clear")]
    CacheClear,

    /// Update theme to latest version
    Update {
        /// Force re-download even if cached
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::New { name, theme, no_git } => {
            commands::new::execute(&name, theme.as_deref(), !no_git)?;
        }
        Commands::Dev { port } => {
            commands::dev::execute(port)?;
        }
        Commands::Build { mode, release } => {
            commands::build::execute(mode, release)?;
        }
        Commands::Theme { command } => {
            commands::theme::execute(command)?;
        }
    }

    Ok(())
}
