use serde_json::{Value, json};

impl super::ISClient {
    pub async fn cache_manager_list(&self) -> Result<Value, String> {
        self.admin_get("/admin/cachemanager").await
    }

    pub async fn cache_manager_get(&self, name: &str) -> Result<Value, String> {
        self.admin_get(&format!("/admin/cachemanager/{name}")).await
    }

    pub async fn cache_manager_create(&self, settings: &Value) -> Result<Value, String> {
        self.admin_post("/admin/cachemanager", settings).await
    }

    pub async fn cache_manager_update(
        &self,
        name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        self.admin_patch(&format!("/admin/cachemanager/{name}"), settings)
            .await
    }

    pub async fn cache_manager_delete(&self, name: &str) -> Result<Value, String> {
        self.admin_delete(&format!("/admin/cachemanager/{name}"))
            .await
    }

    pub async fn cache_reset(&self, cache_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.cache:resetCache",
            &json!({"cacheName": cache_name}),
        )
        .await
    }

    // ── REST Admin helpers ────────────────────────────────────────────

    pub(crate) async fn admin_get(&self, path: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url(path))
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

    pub(crate) async fn admin_post(&self, path: &str, payload: &Value) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url(path))
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

    pub(crate) async fn admin_patch(&self, path: &str, payload: &Value) -> Result<Value, String> {
        let r = self
            .client
            .patch(self.url(path))
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

    pub(crate) async fn admin_delete(&self, path: &str) -> Result<Value, String> {
        let r = self
            .client
            .delete(self.url(path))
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

    #[allow(dead_code)]
    pub(crate) async fn admin_put(&self, path: &str, payload: &Value) -> Result<Value, String> {
        let r = self
            .client
            .put(self.url(path))
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
