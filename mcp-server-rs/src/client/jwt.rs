use serde_json::{Value, json};

impl super::ISClient {
    pub async fn jwt_issuer_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jwt:listIssuers").await
    }

    pub async fn jwt_issuer_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jwt:addIssuer", settings).await
    }

    pub async fn jwt_issuer_get(&self, name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.jwt:getIssuer", &json!({"issuerName": name}))
            .await
    }

    pub async fn jwt_issuer_delete(&self, name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.jwt:removeIssuer", &json!({"issuerName": name}))
            .await
    }

    pub async fn jwt_settings_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jwt:getGlobalSettings").await
    }

    pub async fn jwt_settings_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jwt:updateGlobalSettings", settings)
            .await
    }
}
