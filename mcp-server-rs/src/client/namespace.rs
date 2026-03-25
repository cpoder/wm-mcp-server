use serde_json::{Value, json};

impl super::ISClient {
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
}
