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
        let config_path = dirs::config_dir().map(|p| p.join("sbr").join("config.toml"));

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

    /// Returns true if the given app name should be skipped.
    pub fn is_excluded(&self, app_name: &str) -> bool {
        self.capture
            .excluded_apps
            .iter()
            .any(|ex| app_name.contains(ex.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_default_ax_enabled() {
        let cfg = default_config();
        assert!(cfg.capture.ax_enabled);
    }

    #[test]
    fn test_default_screenshot_disabled() {
        let cfg = default_config();
        assert!(!cfg.capture.screenshot_enabled);
    }

    #[test]
    fn test_default_poll_interval() {
        let cfg = default_config();
        assert_eq!(cfg.capture.poll_interval_ms, 1000);
    }

    #[test]
    fn test_excluded_app_exact_match() {
        let cfg = default_config();
        assert!(cfg.is_excluded("1Password"));
        assert!(cfg.is_excluded("Keychain Access"));
    }

    #[test]
    fn test_excluded_app_partial_match() {
        let cfg = default_config();
        assert!(cfg.is_excluded("1Password 7 - Password Manager"));
    }

    #[test]
    fn test_non_excluded_app() {
        let cfg = default_config();
        assert!(!cfg.is_excluded("Safari"));
        assert!(!cfg.is_excluded("Slack"));
        assert!(!cfg.is_excluded("Notes"));
    }

    #[test]
    fn test_parse_toml_config() {
        let toml_str = r#"
[capture]
ax_enabled = false
screenshot_enabled = true
poll_interval_ms = 500
max_tree_depth = 5
min_text_length = 20
excluded_apps = ["Bitwarden", "Keychain Access"]
"#;
        let cfg: Config = toml::from_str(toml_str).expect("should parse");
        assert!(!cfg.capture.ax_enabled);
        assert!(cfg.capture.screenshot_enabled);
        assert_eq!(cfg.capture.poll_interval_ms, 500);
        assert_eq!(cfg.capture.max_tree_depth, 5);
        assert_eq!(cfg.capture.min_text_length, 20);
        assert_eq!(
            cfg.capture.excluded_apps,
            vec!["Bitwarden", "Keychain Access"]
        );
    }

    #[test]
    fn test_custom_excluded_apps() {
        let toml_str = r#"
[capture]
ax_enabled = true
screenshot_enabled = false
poll_interval_ms = 1000
max_tree_depth = 8
min_text_length = 10
excluded_apps = ["MyBankApp"]
"#;
        let cfg: Config = toml::from_str(toml_str).expect("should parse");
        assert!(cfg.is_excluded("MyBankApp"));
        assert!(!cfg.is_excluded("1Password"));
    }
}
