use serde_json::{Value, json};

impl super::ISClient {
    pub async fn quiesce_status(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.quiesce:getCurrentMode").await
    }

    pub async fn quiesce_enable(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.quiesce:setQuiesceMode", settings)
            .await
    }

    pub async fn quiesce_disable(&self) -> Result<Value, String> {
        self.invoke_post("wm.server.quiesce:setActiveMode", &json!({}))
            .await
    }
}
