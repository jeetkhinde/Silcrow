use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme manifest structure (theme.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    pub theme: ThemeInfo,
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub requires: ThemeRequirements,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeRequirements {
    #[serde(default)]
    pub rhtmx: Option<String>,
}

impl ThemeManifest {
    /// Parse theme manifest from TOML string
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Load theme manifest from file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&content)?)
    }
}
