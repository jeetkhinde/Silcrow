use anyhow::Result;
use std::path::PathBuf;

/// Theme source type
#[derive(Debug, Clone)]
pub enum ThemeSource {
    Local(PathBuf),
    Git { url: String, branch: Option<String> },
}

/// Theme registry for discovering and managing themes
pub struct ThemeRegistry {
    // TODO: Implement theme registry
    // - Local theme discovery
    // - Git theme cloning
    // - Theme version management
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self {}
    }

    /// List available themes
    pub fn list_themes(&self) -> Result<Vec<String>> {
        // TODO: Discover themes from:
        // - Local directories
        // - Git repositories
        // - Official theme registry
        Ok(vec![])
    }

    /// Resolve theme source from string
    pub fn resolve_source(source: &str) -> Result<ThemeSource> {
        if source.starts_with("http://") || source.starts_with("https://") || source.ends_with(".git") {
            Ok(ThemeSource::Git {
                url: source.to_string(),
                branch: None,
            })
        } else {
            Ok(ThemeSource::Local(PathBuf::from(source)))
        }
    }

    /// Install theme from source
    pub fn install_theme(&self, _source: ThemeSource) -> Result<()> {
        // TODO: Implement theme installation
        // - Clone git repo or copy local directory
        // - Verify theme structure
        // - Cache in ~/.rhtmx/themes/
        Ok(())
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
