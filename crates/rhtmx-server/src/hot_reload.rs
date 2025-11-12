use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// Type of file change that occurred
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Template,
    Component,
    SourceCode,
}

/// Represents a file change event
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
}

/// Hot reload watcher that monitors file system changes
pub struct HotReloadWatcher {
    tx: broadcast::Sender<FileChange>,
    _watcher: notify::RecommendedWatcher,
}

impl HotReloadWatcher {
    /// Create a new hot reload watcher
    pub fn new(watch_paths: Vec<PathBuf>) -> Result<Self> {
        let (tx, _) = broadcast::channel(100);
        let tx_clone = tx.clone();

        // Create file watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Only process modify and create events
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            // Determine change type based on file path
                            let path_str = path.to_str().unwrap_or("");

                            let change_type =
                                if path_str.contains("pages/") || path_str.contains("pages\\") {
                                    ChangeType::Template
                                } else if path_str.contains("components/")
                                    || path_str.contains("components\\")
                                {
                                    ChangeType::Component
                                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                                    ChangeType::SourceCode
                                } else {
                                    continue; // Skip other files
                                };

                            info!("📝 File changed: {:?} ({:?})", path, change_type);

                            let file_change = FileChange {
                                path: path.clone(),
                                change_type,
                            };

                            // Broadcast change event (ignore if no receivers)
                            let _ = tx_clone.send(file_change);
                        }
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        })?;

        // Watch all specified paths
        for path in watch_paths {
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)?;
                info!("👀 Watching: {:?}", path);
            } else {
                warn!("⚠️  Path does not exist: {:?}", path);
            }
        }

        Ok(Self {
            tx,
            _watcher: watcher,
        })
    }

    /// Subscribe to file change events
    pub fn subscribe(&self) -> broadcast::Receiver<FileChange> {
        self.tx.subscribe()
    }
}

/// Create a hot reload watcher for the rhtmx application
pub fn create_watcher() -> Result<HotReloadWatcher> {
    let watch_paths = vec![
        PathBuf::from("pages"),
        PathBuf::from("components"),
        PathBuf::from("src"),
    ];

    HotReloadWatcher::new(watch_paths)
}
