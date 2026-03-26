use serde_json::{Value, json};

impl super::ISClient {
    pub async fn alert_status(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.alert:alertingStatus").await
    }

    pub async fn alert_enable(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.alert:enableNotifiers", &json!({}))
            .await
    }

    pub async fn alert_disable(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.alert:disableAllNotifiers", &json!({}))
            .await
    }
}
