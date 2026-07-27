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
    /// Extra `Host` authorities accepted in HTTP mode, on top of the loopback
    /// defaults. Set via WM_ALLOWED_HOSTS (comma-separated), e.g.
    /// "mcp.example.com,mcp.example.com:8080". Empty = loopback only.
    /// A single `*` disables Host validation entirely.
    pub allowed_hosts: Vec<String>,
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
            allowed_hosts: Self::load_allowed_hosts(),
        })
    }

    fn load_scopes() -> Vec<String> {
        Self::load_csv_env("WM_SCOPES")
    }

    fn load_allowed_hosts() -> Vec<String> {
        Self::load_csv_env("WM_ALLOWED_HOSTS")
    }

    fn load_csv_env(var: &str) -> Vec<String> {
        std::env::var(var)
            .ok()
            .map(|s| parse_csv(&s))
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
            allowed_hosts: Self::load_allowed_hosts(),
        }
    }
}

/// Split a comma-separated env var into trimmed, lowercased, non-empty entries.
pub fn parse_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Merge operator-supplied `Host` authorities into the transport's loopback
/// defaults, preserving order and dropping duplicates.
///
/// Returns `None` when the operator asked for validation to be switched off
/// entirely (a single `*` entry), which reopens the DNS-rebinding hole patched
/// by RUSTSEC-2026-0189 and is therefore logged as a warning by the caller.
///
/// Extra hosts are *added* to the defaults rather than replacing them, so
/// loopback health checks keep working once a public hostname is configured.
pub fn resolve_allowed_hosts(defaults: &[String], configured: &[String]) -> Option<Vec<String>> {
    if configured.iter().any(|h| h == "*") {
        return None;
    }
    let mut hosts: Vec<String> = Vec::with_capacity(defaults.len() + configured.len());
    for host in defaults.iter().chain(configured) {
        if !hosts.iter().any(|seen| seen == host) {
            hosts.push(host.clone());
        }
    }
    Some(hosts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Vec<String> {
        ["localhost", "127.0.0.1", "::1"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parse_csv_trims_lowercases_and_drops_empties() {
        assert_eq!(
            parse_csv(" Develop , MONITOR ,, deploy,"),
            vec!["develop", "monitor", "deploy"]
        );
    }

    #[test]
    fn parse_csv_of_blank_input_is_empty() {
        assert!(parse_csv("").is_empty());
        assert!(parse_csv("  ,  , ").is_empty());
    }

    #[test]
    fn no_configured_hosts_keeps_loopback_defaults() {
        assert_eq!(resolve_allowed_hosts(&defaults(), &[]), Some(defaults()));
    }

    #[test]
    fn configured_hosts_are_added_not_substituted() {
        // Regression guard: replacing the defaults would break loopback health
        // checks the moment an operator sets a public hostname.
        let configured = vec!["mcp.example.com".to_string()];
        let merged = resolve_allowed_hosts(&defaults(), &configured).unwrap();
        for host in defaults() {
            assert!(merged.contains(&host), "lost loopback default {host}");
        }
        assert!(merged.contains(&"mcp.example.com".to_string()));
    }

    #[test]
    fn overlapping_hosts_are_deduplicated_in_order() {
        let configured = vec!["localhost".to_string(), "mcp.example.com".to_string()];
        assert_eq!(
            resolve_allowed_hosts(&defaults(), &configured).unwrap(),
            vec!["localhost", "127.0.0.1", "::1", "mcp.example.com"]
        );
    }

    #[test]
    fn wildcard_disables_validation_even_alongside_other_entries() {
        assert_eq!(resolve_allowed_hosts(&defaults(), &["*".to_string()]), None);
        let mixed = vec!["mcp.example.com".to_string(), "*".to_string()];
        assert_eq!(resolve_allowed_hosts(&defaults(), &mixed), None);
    }

    #[test]
    fn wildcard_is_only_honoured_as_a_whole_entry() {
        // "*.example.com" is not a supported wildcard syntax; it must not be
        // mistaken for the disable-everything switch.
        let hosts = vec!["*.example.com".to_string()];
        assert!(resolve_allowed_hosts(&defaults(), &hosts).is_some());
    }
}
