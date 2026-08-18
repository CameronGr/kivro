//! global configuration
//!
//! ~/.config/kivro/config.toml
//! %APPDATA%\kivro\
//! ~/Library/Application Support/kivro
//! overide with KIVRO_CONFIG_DIR var

use std::path::{Path, PathBuf};

use kivro_core::{Error, Result};
use serde::{Deserialize, Serialize};

/// Environment variable overriding the configuration directory
pub const CONFIG_DIR_ENV: &str = "KIVRO_CONFIG_DIR";
/// Configuration filename
pub const CONFIG_FILENAME: &str = "config.toml";

/// parsed global configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// fallback defaults
    pub defaults: Defaults,
    /// presentation preferences
    pub ui: Ui,
    /// sStorage preferences
    pub storage: Storage,
}

/// `[defaults]`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Defaults {
    /// Environment to use when neither the CLI nor the manifest selects one
    pub environment: Option<String>,
}

/// `[ui]`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ui {
    /// colourise output when attached to a terminal
    pub color: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self { color: true }
    }
}

/// `[storage]`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Storage {
    /// application namespace used as the first level of the keyring namespace
    pub namespace: String,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            namespace: kivro_core::DEFAULT_APP_NAMESPACE.to_string(),
        }
    }
}

impl Config {
    /// configuration directory for this platform
    pub fn directory() -> Result<PathBuf> {
        if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
            return Ok(PathBuf::from(dir));
        }
        dirs::config_dir()
            .map(|d| d.join("kivro"))
            .ok_or_else(|| Error::Other("cannot determine a configuration directory".into()))
    }

    /// path of the config
    pub fn path() -> Result<PathBuf> {
        Ok(Self::directory()?.join(CONFIG_FILENAME))
    }

    /// load configuration
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        Self::load_from(&path)
    }

    /// load configuration from path
    pub fn load_from(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| Error::Config {
                path: path.to_path_buf(),
                message: e.message().to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::io("read configuration", path, e)),
        }
    }

    /// write configuration
    pub fn save(&self) -> Result<PathBuf> {
        let dir = Self::directory()?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| Error::io("create configuration directory", &dir, e))?;
        let path = dir.join(CONFIG_FILENAME);
        let text = toml::to_string_pretty(self).map_err(|e| Error::Config {
            path: path.clone(),
            message: e.to_string(),
        })?;
        std::fs::write(&path, text).map_err(|e| Error::io("write configuration", &path, e))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_configuration_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config::load_from(&dir.path().join("nope.toml")).unwrap();
        assert!(cfg.ui.color);
        assert_eq!(cfg.storage.namespace, kivro_core::DEFAULT_APP_NAMESPACE);
        assert!(cfg.defaults.environment.is_none());
    }

    #[test]
    fn partial_configuration_keeps_other_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[defaults]\nenvironment = \"staging\"\n").unwrap();
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(cfg.defaults.environment.as_deref(), Some("staging"));
        assert!(cfg.ui.color);
    }

    #[test]
    fn invalid_configuration_is_reported_with_its_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not toml =\n").unwrap();
        let err = Config::load_from(&path).unwrap_err();
        assert_eq!(err.kind(), "config_invalid");
    }
}
