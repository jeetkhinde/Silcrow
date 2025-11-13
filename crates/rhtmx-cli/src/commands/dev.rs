use anyhow::Result;
use colored::Colorize;
use std::env;
use crate::theme::ThemeManager;

#[cfg(feature = "dev-server")]
pub fn execute(port: u16) -> Result<()> {
    use crate::dev::server::start_dev_server;
    use crate::dev::watcher::FileWatcher;

    println!("{}", "Preparing development environment...".green().bold());
    println!();

    // Load and merge theme with user files
    let current_dir = env::current_dir()?;
    let manager = ThemeManager::new(&current_dir);

    println!("{}", "  ⚙  Loading theme...".cyan());
    manager.load_and_merge(false)?;

    // Get merged path
    let merged_path = manager.merged_path().to_path_buf();

    // Start async runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            // Start file watcher
            let watcher = FileWatcher::new(current_dir);
            if let Err(e) = watcher.watch().await {
                eprintln!("⚠ Failed to start file watcher: {}", e);
                eprintln!("  Continuing without hot reload...");
            }

            // Start dev server
            start_dev_server(&merged_path, port).await
        })
}

#[cfg(not(feature = "dev-server"))]
pub fn execute(_port: u16) -> Result<()> {
    println!("{}", "⚠ Dev server not available".yellow());
    println!();
    println!("The dev server requires the 'dev-server' feature.");
    println!("Rebuild with: cargo build --features dev-server");
    Ok(())
}
