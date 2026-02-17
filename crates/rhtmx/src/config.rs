use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub routing: RoutingConfig,

    #[serde(default)]
    pub dev: DevConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_host")]
    pub host: String,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    #[serde(default = "default_pages_dir")]
    pub pages_dir: String,

    #[serde(default = "default_components_dir")]
    pub components_dir: String,

    #[serde(default = "default_true")]
    pub case_insensitive: bool,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    #[serde(default = "default_true")]
    pub hot_reload: bool,
}

fn default_port() -> u16 { 3000 }
fn default_host() -> String { "127.0.0.1".to_string() }
fn default_pages_dir() -> String { "pages".to_string() }
fn default_components_dir() -> String { "components".to_string() }
fn default_true() -> bool { true }

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000, host: "127.0.0.1".to_string() }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            pages_dir: "pages".to_string(),
            components_dir: "components".to_string(),
            case_insensitive: true,
        }
    }
}

impl Default for DevConfig {
    fn default() -> Self {
        Self { hot_reload: true }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        if content.trim().is_empty() {
            return Ok(Config::default());
        }
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    }

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
        assert!(config.routing.case_insensitive);
        assert_eq!(config.routing.pages_dir, "pages");
    }

    #[test]
    fn test_custom_config() {
        let toml = r#"
            [routing]
            pages_dir = "app"
            components_dir = "ui"
            case_insensitive = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.routing.pages_dir, "app");
        assert!(!config.routing.case_insensitive);
    }
}
