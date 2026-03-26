//! Configuration for one or more webMethods IS instances.
//!
//! Load order:
//! 1. If `WM_CONFIG` env var is set, read that JSON file.
//! 2. Otherwise, build a single-instance config from `WM_IS_*` env vars.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct InstanceConfig {
    pub url: String,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default = "default_password")]
    pub password: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_user() -> String {
    "Administrator".into()
}
fn default_password() -> String {
    "manage".into()
}
fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    pub instances: HashMap<String, InstanceConfig>,
    /// Optional default instance name. If omitted, the first key is used.
    pub default: Option<String>,
}

pub struct AppConfig {
    pub instances: HashMap<String, InstanceConfig>,
    pub default_instance: String,
    /// Tool scopes to expose. Empty = all tools. Set via WM_SCOPES env var (comma-separated).
    /// Valid scopes: admin, develop, adapters, messaging, monitor, deploy, network, readonly
    pub scopes: Vec<String>,
}

impl AppConfig {
    /// Load config from `WM_CONFIG` file or fall back to `WM_IS_*` env vars.
    pub fn load() -> Result<Self, String> {
        if let Ok(path) = std::env::var("WM_CONFIG") {
            Self::from_file(&path)
        } else {
            Ok(Self::from_env())
        }
    }

    fn from_file(path: &str) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Cannot read {path}: {e}"))?;
        let file: FileConfig =
            serde_json::from_str(&content).map_err(|e| format!("Invalid config JSON: {e}"))?;

        if file.instances.is_empty() {
            return Err("Config file has no instances defined".into());
        }

        let default_instance = file
            .default
            .or_else(|| file.instances.keys().next().cloned())
            .unwrap();

        if !file.instances.contains_key(&default_instance) {
            return Err(format!(
                "Default instance '{}' not found in instances",
                default_instance
            ));
        }

        Ok(Self {
            instances: file.instances,
            default_instance,
            scopes: Self::load_scopes(),
        })
    }

    fn load_scopes() -> Vec<String> {
        std::env::var("WM_SCOPES")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn from_env() -> Self {
        let instance = InstanceConfig {
            url: std::env::var("WM_IS_URL").unwrap_or_else(|_| "http://localhost:5555".into()),
            user: std::env::var("WM_IS_USER").unwrap_or_else(|_| "Administrator".into()),
            password: std::env::var("WM_IS_PASSWORD").unwrap_or_else(|_| "manage".into()),
            timeout: std::env::var("WM_IS_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        };

        let name = std::env::var("WM_IS_NAME").unwrap_or_else(|_| "default".into());
        let mut instances = HashMap::new();
        instances.insert(name.clone(), instance);

        Self {
            instances,
            default_instance: name,
            scopes: Self::load_scopes(),
        }
    }
}
