use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use dirs::config_dir;
use serde::Deserialize;

/// User-level configuration loaded from `~/.config/frodo/config.toml` (platform-specific).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct Config {
    /// Override for data directory (encrypted store).
    pub data_dir: Option<PathBuf>,
    /// Agent provider config (future expansion).
    pub openai: Option<OpenAiConfig>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
}

/// Load config from the default path; if missing, return defaults.
pub fn load() -> Result<Config> {
    let path = default_path()?;
    load_from_path(path)
}

/// Load config from a given path; if missing or empty, return defaults.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(path)?;
    if contents.trim().is_empty() {
        return Ok(Config::default());
    }
    let cfg: Config = toml::from_str(&contents)?;
    Ok(cfg)
}

/// Resolve the default config path (platform aware).
pub fn default_path() -> Result<PathBuf> {
    let base = config_dir().ok_or_else(|| color_eyre::eyre::eyre!("no config dir available"))?;
    Ok(base.join("frodo").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_default_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = load_from_path(dir.path().join("config.toml")).expect("load");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn parses_custom_config() {
        let contents = r#"
            data_dir = "/tmp/frodo-data"
            [openai]
            api_key = "secret"
            model = "gpt-4.2"
            endpoint = "https://api.openai.com/v1"
        "#;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        fs::write(&path, contents).expect("write temp config");

        let cfg = load_from_path(&path).expect("load");
        assert_eq!(
            cfg,
            Config {
                data_dir: Some(PathBuf::from("/tmp/frodo-data")),
                openai: Some(OpenAiConfig {
                    api_key: Some("secret".into()),
                    model: Some("gpt-4.2".into()),
                    endpoint: Some("https://api.openai.com/v1".into()),
                }),
            }
        );
    }
}
