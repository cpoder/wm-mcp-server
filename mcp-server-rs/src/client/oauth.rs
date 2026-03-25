use serde_json::{Value, json};

impl super::ISClient {
    pub async fn oauth_settings_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.oauth:getOAuthSettings").await
    }

    pub async fn oauth_settings_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.oauth:setOAuthSettings", settings)
            .await
    }

    pub async fn oauth_client_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.oauth:listClientRegistrations")
            .await
    }

    pub async fn oauth_client_register(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.oauth:registerClient", settings)
            .await
    }

    pub async fn oauth_client_delete(&self, client_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.oauth:removeClientRegistration",
            &json!({"client_id": client_id}),
        )
        .await
    }

    pub async fn oauth_scope_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.oauth:listScopes").await
    }

    pub async fn oauth_scope_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.oauth:putScope", settings).await
    }

    pub async fn oauth_scope_remove(&self, name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.oauth:removeScope", &json!({"name": name}))
            .await
    }

    pub async fn oauth_token_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.oauth:listAccessTokens").await
    }
}
