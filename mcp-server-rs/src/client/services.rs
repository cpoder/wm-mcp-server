use serde_json::{Value, json};
use std::collections::HashMap;

impl super::ISClient {
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
}
