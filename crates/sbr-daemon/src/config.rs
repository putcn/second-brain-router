use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub capture: CaptureConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaptureConfig {
    pub ax_enabled: bool,
    pub screenshot_enabled: bool,
    pub poll_interval_ms: u64,
    pub excluded_apps: Vec<String>,
    pub max_tree_depth: usize,
    pub min_text_length: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            capture: CaptureConfig {
                ax_enabled: true,
                screenshot_enabled: false,
                poll_interval_ms: 1000,
                excluded_apps: vec![
                    "1Password".into(),
                    "Keychain Access".into(),
                    "System Preferences".into(),
                ],
                max_tree_depth: 8,
                min_text_length: 10,
            },
        }
    }
}

impl Config {
    pub fn load_or_default() -> Self {
        let config_path = dirs::config_dir()
            .map(|p| p.join("sbr").join("config.toml"));

        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(cfg) = toml::from_str::<Config>(&content) {
                        return cfg;
                    }
                }
            }
        }

        Config::default()
    }
}
