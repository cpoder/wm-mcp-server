use serde_json::Value;

impl super::ISClient {
    pub async fn password_policy_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.access:getPasswordExpirySettings")
            .await
    }

    pub async fn password_policy_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.access:updateExpirySettings", settings)
            .await
    }
}
