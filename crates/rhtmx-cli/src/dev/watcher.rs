#[cfg(feature = "dev-server")]
use anyhow::Result;
#[cfg(feature = "dev-server")]
use colored::Colorize;
#[cfg(feature = "dev-server")]
use notify::{Event, EventKind, RecursiveMode, Watcher};
#[cfg(feature = "dev-server")]
use std::path::{Path, PathBuf};
#[cfg(feature = "dev-server")]
use std::sync::Arc;
#[cfg(feature = "dev-server")]
use tokio::sync::RwLock;

#[cfg(feature = "dev-server")]
use crate::theme::ThemeManager;

/// File change event
#[cfg(feature = "dev-server")]
#[derive(Debug, Clone)]
pub enum ChangeType {
    UserFile,
    ThemeFile,
}

/// Watch for file changes and trigger re-merge
#[cfg(feature = "dev-server")]
pub struct FileWatcher {
    project_root: PathBuf,
    theme_manager: Arc<RwLock<ThemeManager>>,
}

#[cfg(feature = "dev-server")]
impl FileWatcher {
    pub fn new(project_root: PathBuf) -> Self {
        let theme_manager = Arc::new(RwLock::new(ThemeManager::new(&project_root)));

        Self {
            project_root,
            theme_manager,
        }
    }

    /// Start watching for file changes
    pub async fn watch(&self) -> Result<()> {
        let project_root = self.project_root.clone();
        let theme_manager = self.theme_manager.clone();

        // Paths to watch
        let watch_paths = vec![
            project_root.join("pages"),
            project_root.join("components"),
            project_root.join("static"),
            project_root.join("rhtmx.toml"),
        ];

        // Check if using local theme and add to watch list
        let config_path = project_root.join("rhtmx.toml");
        let theme_path = if config_path.exists() {
            if let Ok(config) = crate::theme::manifest::ProjectConfig::from_file(&config_path) {
                if let Some(theme_config) = &config.theme {
                    match &theme_config.source {
                        crate::theme::manifest::ThemeSource::Local { path } => {
                            Some(path.clone())
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let theme_path_clone = theme_path.clone();

        // Create file watcher
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only process modify and create events
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in &event.paths {
                        // Ignore hidden files and directories
                        if path.to_str().map_or(false, |s| {
                            s.contains("/.") || s.contains("\\.") ||
                            s.contains(".rhtmx/") || s.contains(".themes/")
                        }) {
                            continue;
                        }

                        let _ = tx.blocking_send(path.clone());
                    }
                }
            }
        })?;

        // Watch all paths
        for path in &watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                println!("  {} Watching: {}", "👀".cyan(), path.display());
            }
        }

        // Watch theme directory if local
        if let Some(theme_dir) = &theme_path {
            if theme_dir.exists() {
                watcher.watch(theme_dir, RecursiveMode::Recursive)?;
                println!("  {} Watching theme: {}", "👀".cyan(), theme_dir.display());
            }
        }

        println!();

        // Spawn task to handle file changes
        tokio::spawn(async move {
            let _watcher = watcher; // Keep watcher alive

            // Debounce changes (wait a bit to batch rapid changes)
            let mut last_merge = std::time::Instant::now();
            let debounce_duration = std::time::Duration::from_millis(300);

            while let Some(path) = rx.recv().await {
                // Check if enough time has passed since last merge
                let now = std::time::Instant::now();
                if now.duration_since(last_merge) < debounce_duration {
                    continue;
                }

                // Determine what changed
                let path_str = path.to_str().unwrap_or("");
                let is_theme_file = theme_path_clone.as_ref()
                    .map_or(false, |tp| path_str.starts_with(tp.to_str().unwrap_or("")));

                let change_type = if is_theme_file {
                    ChangeType::ThemeFile
                } else {
                    ChangeType::UserFile
                };

                // Log the change
                match change_type {
                    ChangeType::UserFile => {
                        println!("{} User file changed: {}", "🔄".yellow(), path.display());
                    }
                    ChangeType::ThemeFile => {
                        println!("{} Theme file changed: {}", "🔄".yellow(), path.display());
                    }
                }

                // Re-merge theme and user files
                println!("{} Re-merging files...", "⚙".cyan());

                let manager = theme_manager.read().await;
                if let Err(e) = manager.load_and_merge(false) {
                    eprintln!("{} Failed to re-merge: {}", "❌".red(), e);
                } else {
                    println!("{} Merge complete - browser will reload", "✓".green());
                }

                last_merge = now;
            }
        });

        Ok(())
    }

    pub fn theme_manager(&self) -> Arc<RwLock<ThemeManager>> {
        self.theme_manager.clone()
    }
}
