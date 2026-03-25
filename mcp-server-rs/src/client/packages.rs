use reqwest::StatusCode;
use serde_json::{Value, json};

impl super::ISClient {
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
}
