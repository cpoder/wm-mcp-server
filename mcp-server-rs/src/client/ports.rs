use serde_json::{Value, json};

impl super::ISClient {
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
}
