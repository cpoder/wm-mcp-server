//! webMethods Integration Server HTTP Client
//!
//! Pure HTTP client for interacting with the IS REST API.
//! All operations use the IS HTTP API - no disk access required.

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::collections::HashMap;

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

    // ── Server Management ──────────────────────────────────────────────

    pub async fn is_running(&self) -> bool {
        match self
            .client
            .get(self.url("/invoke/wm.server.packages/packageList"))
            .send()
            .await
        {
            Ok(r) => r.status() == StatusCode::OK,
            Err(_) => false,
        }
    }

    pub async fn get_server_status(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.admin/getServerStatus"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn shutdown(&self, bounce: bool) -> Result<Value, String> {
        let mut url = self.url("/invoke/wm.server.admin/shutdown");
        if bounce {
            url.push_str("?bounce=yes");
        }
        let r = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "shutdown initiated", "bounce": bounce}))
    }

    // ── Package Management ─────────────────────────────────────────────

    pub async fn package_list(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.packages/packageList"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn package_create(&self, package_name: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.packages/packageCreate"))
            .json(&json!({"package": package_name}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        // Auto-activate
        let _ = self
            .client
            .get(self.url(&format!(
                "/invoke/wm.server.packages/packageActivate?package={}",
                package_name
            )))
            .send()
            .await;
        Ok(json!({"status": "created", "package": package_name}))
    }

    pub async fn package_reload(&self, package_name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!(
                "/invoke/wm.server.packages/packageReload?package={}",
                package_name
            )))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "reloaded", "package": package_name}))
    }

    pub async fn package_enable(&self, package_name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!(
                "/invoke/wm.server.packages/packageEnable?package={}",
                package_name
            )))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "enabled", "package": package_name}))
    }

    pub async fn package_disable(&self, package_name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(&format!(
                "/invoke/wm.server.packages/packageDisable?package={}",
                package_name
            )))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "disabled", "package": package_name}))
    }

    // ── Namespace / Node Management ────────────────────────────────────

    pub async fn node_list(&self, package: &str, interface: &str) -> Result<Value, String> {
        let mut params = vec![("package", package)];
        if !interface.is_empty() {
            params.push(("interface", interface));
        }
        let r = self
            .client
            .get(self.url("/invoke/wm.server.ns/getNodeList"))
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn node_get(&self, name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.ns/getNode"))
            .query(&[("name", name)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn node_delete(&self, name: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/deleteNode"))
            .json(&json!({"node_nsName": name}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "deleted", "node": name}))
    }

    pub async fn folder_create(&self, package: &str, folder_path: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/makeNode"))
            .json(&json!({
                "node_type": "interface",
                "node_nsName": folder_path,
                "node_pkg": package,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "created", "folder": folder_path, "package": package}))
    }

    // ── Service Management via putNode ─────────────────────────────────

    pub async fn put_node(&self, node_data: &Value) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/putNode"))
            .json(node_data)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "ok", "response": truncated}))
    }

    pub async fn service_create(&self, package: &str, service_path: &str) -> Result<Value, String> {
        let (interface_part, service_name) = if let Some(pos) = service_path.rfind(':') {
            (&service_path[..pos], &service_path[pos + 1..])
        } else {
            ("", service_path)
        };

        let mut payload: HashMap<&str, &str> = HashMap::new();
        payload.insert("service", service_name);
        payload.insert("package", package);
        payload.insert("serviceType", "flow");
        if !interface_part.is_empty() {
            payload.insert("interface", interface_part);
        }

        let r = self
            .client
            .post(self.url("/invoke/wm.server.services/serviceAdd"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "created", "service": service_path, "package": package}))
    }

    pub async fn service_invoke(
        &self,
        service_path: &str,
        inputs: Option<&Value>,
    ) -> Result<Value, String> {
        let url = self.url(&format!("/invoke/{}", service_path));
        let r = if let Some(body) = inputs {
            self.client.post(&url).json(body).send().await
        } else {
            self.client.get(&url).send().await
        }
        .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        if text.trim().is_empty() {
            Ok(json!({"status": "invoked"}))
        } else {
            serde_json::from_str(&text).map_err(|e| e.to_string())
        }
    }

    // ── Document Type Management ───────────────────────────────────────

    pub async fn document_type_create(
        &self,
        package: &str,
        doc_path: &str,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/makeNode"))
            .json(&json!({
                "node_type": "record",
                "node_nsName": doc_path,
                "node_pkg": package,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "created", "document": doc_path, "package": package}))
    }

    // ── Port / Listener Management ─────────────────────────────────────

    pub async fn port_list(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.net.listeners/listListeners"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn port_factory_list(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.net.listeners/listFactories"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn port_get(&self, port_key: &str, pkg: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/getListener"))
            .json(&json!({"listenerKey": port_key, "pkg": pkg}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn port_add(&self, settings: &Value) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/addListener"))
            .json(settings)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let data: Value = serde_json::from_str(&text).unwrap_or(json!({}));
        Ok(json!({
            "status": "created",
            "message": data.get("message").and_then(|v| v.as_str()).unwrap_or(""),
            "listenerKey": data.get("listenerKey").and_then(|v| v.as_str()).unwrap_or(""),
        }))
    }

    pub async fn port_update(
        &self,
        listener_key: &str,
        pkg: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = settings.clone();
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("listenerKey".into(), json!(listener_key));
            obj.insert("pkg".into(), json!(pkg));
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/updateListener"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "updated", "listener": listener_key}))
    }

    pub async fn port_enable(&self, port_key: &str, pkg: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/enableListener"))
            .json(&json!({"listenerKey": port_key, "pkg": pkg}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "enabled", "listener": port_key}))
    }

    pub async fn port_disable(&self, port_key: &str, pkg: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/disableListener"))
            .json(&json!({"listenerKey": port_key, "pkg": pkg}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "disabled", "listener": port_key}))
    }

    pub async fn port_delete(&self, port_key: &str, pkg: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.net.listeners/deleteListener"))
            .json(&json!({"listenerKey": port_key, "pkg": pkg}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "deleted", "listener": port_key}))
    }

    // ── Adapter Connection Management ──────────────────────────────────

    pub async fn adapter_type_list(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.art.admin:retrieveAdapterTypesList"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn adapter_connection_metadata(
        &self,
        adapter_type: &str,
        factory_type: &str,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.connection:fetchConnectionMetadata"))
            .json(&json!({
                "adapterTypeName": adapter_type,
                "connectionFactoryType": factory_type,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn adapter_connection_list(&self) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.art.admin.connection:listAllResources"))
            .send()
            .await
            .map_err(|e| format!("{e}"))?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn adapter_connection_create(
        &self,
        connection_alias: &str,
        package_name: &str,
        adapter_type: &str,
        connection_factory_type: &str,
        connection_settings: &Value,
        connection_manager_settings: &Value,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.connection:createConnectionNode"))
            .json(&json!({
                "connectionAlias": connection_alias,
                "packageName": package_name,
                "adapterTypeName": adapter_type,
                "connectionFactoryType": connection_factory_type,
                "connectionSettings": connection_settings,
                "connectionManagerSettings": connection_manager_settings,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "connection": connection_alias, "response": truncated}))
    }

    pub async fn adapter_connection_enable(&self, connection_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.connection:enableConnection"))
            .json(&json!({"connectionAlias": connection_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "enabled", "connection": connection_alias}))
    }

    pub async fn adapter_connection_disable(
        &self,
        connection_alias: &str,
    ) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.connection:disableConnection"))
            .json(&json!({"connectionAlias": connection_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "disabled", "connection": connection_alias}))
    }

    pub async fn adapter_connection_state(&self, connection_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.connection:queryConnectionState"))
            .json(&json!({"connectionAlias": connection_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    // ── Adapter Listener Management ────────────────────────────────────

    pub async fn adapter_listener_list(&self, adapter_type: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:listAdapterListeners"))
            .json(&json!({"adapterTypeName": adapter_type}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn adapter_listener_create(
        &self,
        listener_alias: &str,
        package_name: &str,
        adapter_type: &str,
        connection_alias: &str,
        listener_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "listenerAlias": listener_alias,
            "packageName": package_name,
            "adapterTypeName": adapter_type,
            "connectionAlias": connection_alias,
        });
        if let Some(settings) = listener_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("listenerSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.listener:createListenerNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "listener": listener_alias, "response": truncated}))
    }

    pub async fn adapter_listener_enable(&self, listener_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:enableListener"))
            .json(&json!({"listenerAlias": listener_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "enabled", "listener": listener_alias}))
    }

    pub async fn adapter_listener_disable(&self, listener_alias: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.listener:disableListener"))
            .json(&json!({"listenerAlias": listener_alias}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        Ok(json!({"status": "disabled", "listener": listener_alias}))
    }

    // ── Adapter Service Management ─────────────────────────────────────

    pub async fn adapter_service_create(
        &self,
        service_name: &str,
        package_name: &str,
        connection_alias: &str,
        service_template: &str,
        adapter_service_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "serviceName": service_name,
            "packageName": package_name,
            "connectionAlias": connection_alias,
            "serviceTemplate": service_template,
        });
        if let Some(settings) = adapter_service_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("adapterServiceSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.service:createAdapterServiceNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "service": service_name, "response": truncated}))
    }

    // ── Adapter Notification Management ────────────────────────────────

    pub async fn adapter_notification_list(&self, adapter_type: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/pub.art.notification:listAdapterPollingNotifications"))
            .json(&json!({"adapterTypeName": adapter_type}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn adapter_notification_create_polling(
        &self,
        notification_name: &str,
        package_name: &str,
        connection_alias: &str,
        notification_template: &str,
        notification_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "notificationName": notification_name,
            "packageName": package_name,
            "connectionAlias": connection_alias,
            "notificationTemplate": notification_template,
        });
        if let Some(settings) = notification_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("notificationSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.notification:createPollingNotificationNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "notification": notification_name, "response": truncated}))
    }

    pub async fn adapter_notification_create_listener(
        &self,
        notification_name: &str,
        package_name: &str,
        listener_alias: &str,
        notification_template: &str,
        notification_settings: Option<&Value>,
    ) -> Result<Value, String> {
        let mut payload = json!({
            "notificationName": notification_name,
            "packageName": package_name,
            "listenerAlias": listener_alias,
            "notificationTemplate": notification_template,
        });
        if let Some(settings) = notification_settings {
            payload
                .as_object_mut()
                .unwrap()
                .insert("notificationSettings".into(), settings.clone());
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.art.dev.notification:createListenerNotificationNode"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        let text = r.text().await.map_err(|e| e.to_string())?;
        let truncated: String = text.chars().take(500).collect();
        Ok(json!({"status": "created", "notification": notification_name, "response": truncated}))
    }

    // ── Streaming (WmStreaming) ────────────────────────────────────────

    pub async fn streaming_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getConnectionAliasReport")
            .await
    }

    pub async fn streaming_connection_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:createConnectionAlias", settings)
            .await
    }

    pub async fn streaming_connection_enable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:enableConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_disable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:disableConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_delete(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:deleteConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_test(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:testConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_providers(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getAvailableProviders")
            .await
    }

    pub async fn streaming_event_source_list(
        &self,
        alias_name: Option<&str>,
    ) -> Result<Value, String> {
        let mut payload = json!({});
        if let Some(name) = alias_name {
            payload
                .as_object_mut()
                .unwrap()
                .insert("aliasName".into(), json!(name));
        }
        self.invoke_post("wm.server.streaming:getEventSourceReport", &payload)
            .await
    }

    pub async fn streaming_event_source_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:createEventSourceFlat", settings)
            .await
    }

    pub async fn streaming_event_source_delete(
        &self,
        alias_name: &str,
        reference_id: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:deleteEventSource",
            &json!({"aliasName": alias_name, "referenceId": reference_id}),
        )
        .await
    }

    pub async fn streaming_trigger_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getTriggerReport")
            .await
    }

    pub async fn streaming_trigger_enable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:enableTriggers", &json!({"name": name}))
            .await
    }

    pub async fn streaming_trigger_disable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:disableTriggers",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_trigger_suspend(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:suspendTriggers",
            &json!({"name": name}),
        )
        .await
    }

    // ── Internal helpers ───────────────────────────────────────────────

    async fn invoke_get(&self, service: &str) -> Result<Value, String> {
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

    async fn invoke_post(&self, service: &str, payload: &Value) -> Result<Value, String> {
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
