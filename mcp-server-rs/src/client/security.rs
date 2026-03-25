use serde_json::Value;

impl super::ISClient {
    pub async fn keystore_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.security.keystore:listKeyStoresAndConfiguredKeyAliases")
            .await
    }

    pub async fn truststore_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.security.keystore:listTrustStores")
            .await
    }

    pub async fn security_settings_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.security:getSettings").await
    }

    pub async fn security_settings_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.security:setSettings", settings)
            .await
    }
}
