//! webMethods Integration Server HTTP Client
//!
//! Pure HTTP client for interacting with the IS REST API.
//! All operations use the IS HTTP API - no disk access required.

mod adapters;
mod auditing;
mod global_vars;
mod jdbc_pools;
mod jms;
mod jndi;
mod monitoring;
mod mqtt;
mod namespace;
mod oauth;
mod packages;
mod ports;
mod remote_servers;
mod scheduler;
mod services;
mod streaming;
mod users;
mod webservices;

use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::{Value, json};

pub struct ISClient {
    base_url: String,
    client: Client,
}

impl ISClient {
    pub fn new(base_url: &str, username: &str, password: &str, timeout_secs: u64) -> Self {
        use base64::{Engine, prelude::BASE64_STANDARD};
        let credentials = format!("{username}:{password}");
        let encoded = BASE64_STANDARD.encode(credentials.as_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {encoded}")).expect("valid header"),
        );

        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // ── Internal helpers ───────────────────────────────────────────────

    pub(crate) async fn invoke_get(&self, service: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!("/invoke/{service}")))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        if text.trim().is_empty() {
            Ok(json!({"status": "ok"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }

    pub(crate) async fn invoke_post(
        &self,
        service: &str,
        payload: &Value,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url(&format!("/invoke/{service}")))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        if text.trim().is_empty() {
            Ok(json!({"status": "ok"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }
}
