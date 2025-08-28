use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub auto_remove_minutes: Option<u64>,
    #[serde(default)]
    pub skip_permission_flags: HashMap<String, String>,
    #[serde(default = "default_env_files")]
    pub env_files: Vec<String>,
    // When true, prefer opening the web UI instead of attaching in terminal
    pub web: Option<bool>,
    // Hostname to use when printing/opening the web UI URL (defaults to "localhost")
    pub web_host: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut default_flags = HashMap::new();
        default_flags.insert(
            "claude".to_string(),
            "--dangerously-skip-permissions".to_string(),
        );
        default_flags.insert("gemini".to_string(), "--yolo".to_string());
        default_flags.insert("qwen".to_string(), "--yolo".to_string());

        Self {
            auto_remove_minutes: Some(60),
            skip_permission_flags: default_flags,
            env_files: default_env_files(),
            web: Some(false),
            web_host: Some("localhost".to_string()),
        }
    }
}

fn default_env_files() -> Vec<String> {
    vec![
        ".env".to_string(),
        ".env.local".to_string(),
        ".env.development.local".to_string(),
        ".env.test.local".to_string(),
        ".env.production.local".to_string(),
    ]
}

fn settings_file_path() -> PathBuf {
    if let Ok(dir) = env::var("CODESANDBOX_CONFIG_HOME") {
        return PathBuf::from(dir).join("settings.json");
    }
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    home.join(".config")
        .join("codesandbox")
        .join("settings.json")
}

pub fn load_settings() -> Result<Settings> {
    let path = settings_file_path();
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str::<Settings>(&data) {
            return Ok(settings);
        }
    }
    Ok(Settings::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_when_missing() {
        let tmp = tempdir().unwrap();
        let original = env::var("CODESANDBOX_CONFIG_HOME").ok();
        env::set_var("CODESANDBOX_CONFIG_HOME", tmp.path());

        let settings = load_settings().unwrap();
        assert_eq!(settings.auto_remove_minutes, Some(60));
        assert_eq!(
            settings
                .skip_permission_flags
                .get("claude")
                .map(String::as_str),
            Some("--dangerously-skip-permissions")
        );
        assert_eq!(
            settings
                .skip_permission_flags
                .get("gemini")
                .map(String::as_str),
            Some("--yolo")
        );
        assert_eq!(
            settings
                .skip_permission_flags
                .get("qwen")
                .map(String::as_str),
            Some("--yolo")
        );
        assert_eq!(
            settings.env_files,
            vec![
                ".env",
                ".env.local",
                ".env.development.local",
                ".env.test.local",
                ".env.production.local",
            ]
        );

        if let Some(val) = original {
            env::set_var("CODESANDBOX_CONFIG_HOME", val);
        } else {
            env::remove_var("CODESANDBOX_CONFIG_HOME");
        }
    }

    #[test]
    fn read_from_file() {
        let tmp = tempdir().unwrap();
        let config_dir = tmp.path();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("settings.json"),
            r#"{ "auto_remove_minutes": 30, "skip_permission_flags": { "claude": "--dangerously-skip-permissions" }, "env_files": [".custom.env"] }"#,
        )
        .unwrap();

        let original = env::var("CODESANDBOX_CONFIG_HOME").ok();
        env::set_var("CODESANDBOX_CONFIG_HOME", config_dir);

        let settings = load_settings().unwrap();
        assert_eq!(settings.auto_remove_minutes, Some(30));
        assert_eq!(
            settings
                .skip_permission_flags
                .get("claude")
                .map(String::as_str),
            Some("--dangerously-skip-permissions")
        );
        assert_eq!(settings.env_files, vec![".custom.env".to_string()]);

        if let Some(val) = original {
            env::set_var("CODESANDBOX_CONFIG_HOME", val);
        } else {
            env::remove_var("CODESANDBOX_CONFIG_HOME");
        }
    }
}
