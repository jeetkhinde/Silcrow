// File: src/config.rs
// Purpose: Configuration parsing from rhtml.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub routing: RoutingConfig,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub dev: DevConfig,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub author: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_workers")]
    pub workers: usize,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Directory containing page files (default: "pages")
    #[serde(default = "default_pages_dir")]
    pub pages_dir: String,

    /// Directory containing component files (default: "components")
    #[serde(default = "default_components_dir")]
    pub components_dir: String,

    /// Whether routes are case-insensitive (default: true)
    #[serde(default = "default_true")]
    pub case_insensitive: bool,

    /// Base path for all routes (e.g., "/app")
    #[serde(default)]
    pub base_path: Option<String>,

    /// Whether to enforce trailing slashes
    #[serde(default = "default_false")]
    pub trailing_slash: bool,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    #[serde(default = "default_static_dir")]
    pub static_dir: String,

    #[serde(default = "default_false")]
    pub minify_html: bool,

    #[serde(default = "default_false")]
    pub minify_css: bool,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    #[serde(default = "default_true")]
    pub hot_reload: bool,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_false")]
    pub open_browser: bool,

    #[serde(default = "default_watch_paths")]
    pub watch_paths: Vec<String>,
}

// Default values
fn default_name() -> String {
    "rhtml-app".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_workers() -> usize {
    4
}

fn default_output_dir() -> String {
    "dist".to_string()
}

fn default_static_dir() -> String {
    "static".to_string()
}

fn default_pages_dir() -> String {
    "pages".to_string()
}

fn default_components_dir() -> String {
    "components".to_string()
}

fn default_watch_paths() -> Vec<String> {
    vec![
        "pages".to_string(),
        "components".to_string(),
        "static".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

// Default implementations
impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            version: default_version(),
            author: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            workers: default_workers(),
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            pages_dir: default_pages_dir(),
            components_dir: default_components_dir(),
            case_insensitive: true, // Default to case-insensitive (most user-friendly)
            base_path: None,
            trailing_slash: false,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            static_dir: default_static_dir(),
            minify_html: false,
            minify_css: false,
        }
    }
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            hot_reload: true,
            port: default_port(),
            open_browser: false,
            watch_paths: default_watch_paths(),
        }
    }
}

impl Config {
    /// Load configuration from rhtml.toml
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // If file doesn't exist or is empty, return default config
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        // If file is empty, return default config
        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        // Parse TOML
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        Ok(config)
    }

    /// Load configuration from default path (./rhtml.toml)
    pub fn load_default() -> Result<Self> {
        Self::load("rhtml.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "127.0.0.1");
        assert!(config.routing.case_insensitive); // Now defaults to true
        assert_eq!(config.routing.pages_dir, "pages");
        assert_eq!(config.routing.components_dir, "components");
    }

    #[test]
    fn test_empty_config() {
        let config = toml::from_str::<Config>("").unwrap_or_default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.routing.pages_dir, "pages");
        assert_eq!(config.routing.components_dir, "components");
    }

    #[test]
    fn test_custom_directories() {
        let toml = r#"
            [routing]
            pages_dir = "app"
            components_dir = "ui"
            case_insensitive = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.routing.pages_dir, "app");
        assert_eq!(config.routing.components_dir, "ui");
        assert!(!config.routing.case_insensitive);
    }
}
