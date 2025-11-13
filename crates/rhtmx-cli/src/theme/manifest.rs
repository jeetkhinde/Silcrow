use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// SSG (Static Site Generation) configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SsgConfig {
    /// Output directory for generated static files
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    /// Dynamic route sources for pre-rendering
    #[serde(default)]
    pub dynamic_routes: Vec<DynamicRouteSource>,
}

fn default_output_dir() -> String {
    "dist".to_string()
}

/// Configuration for dynamic route data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRouteSource {
    /// Route pattern (e.g., "/posts/[slug]")
    pub pattern: String,
    /// Source glob pattern for content files (e.g., "content/posts/*.md")
    pub source: String,
    /// Optional field to extract from filename for slug (default: filename without extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug_field: Option<String>,
}

/// Project configuration with theme reference (from rhtmx.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectInfo,
    #[serde(default)]
    pub theme: Option<ThemeConfig>,
    #[serde(default)]
    pub ssg: Option<SsgConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Theme configuration in user's rhtmx.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name (for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Theme source
    pub source: ThemeSource,
}

/// Theme source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThemeSource {
    /// Git repository
    Git {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
    },
    /// Local path
    Local {
        path: PathBuf,
    },
    /// Registry (future)
    Registry {
        name: String,
        version: String,
    },
}

/// Theme manifest (theme.toml inside theme directory)
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

impl ProjectConfig {
    /// Load project config from rhtmx.toml
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Check if project uses a theme
    pub fn has_theme(&self) -> bool {
        self.theme.is_some()
    }

    /// Get theme name for caching
    pub fn theme_name(&self) -> Option<String> {
        self.theme.as_ref().and_then(|t| {
            t.name.clone().or_else(|| {
                // Derive name from source
                match &t.source {
                    ThemeSource::Git { url, .. } => {
                        url.split('/').last().map(|s| s.trim_end_matches(".git").to_string())
                    }
                    ThemeSource::Local { path } => {
                        path.file_name().and_then(|n| n.to_str()).map(String::from)
                    }
                    ThemeSource::Registry { name, .. } => Some(name.clone()),
                }
            })
        })
    }
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
