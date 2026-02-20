use serde::Deserialize;
// silcrow/crates/silcrow/src/config.rs — Silcrow project configuration, loaded from `silcrow.toml`
/// Silcrow project configuration, loaded from `silcrow.toml`.
#[derive(Debug, Deserialize)]
pub struct SilcrowConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_port() -> u16 {
    3000
}
fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_name() -> String {
    "silcrow-app".into()
}

impl Default for SilcrowConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            version: String::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

impl SilcrowConfig {
    /// Load from `silcrow.toml` in the current directory.
    /// Returns default config if the file doesn't exist.
    pub fn load() -> Self {
        Self::load_from("silcrow.toml")
    }

    /// Load from a specific path.
    /// Returns default config if the file doesn't exist.
    pub fn load_from(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("[silcrow] Warning: failed to parse {}: {}", path, e);
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}
