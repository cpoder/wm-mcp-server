use serde_json::{Value, json};

impl super::ISClient {
    pub async fn saml_issuer_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.saml:listIssuers").await
    }

    pub async fn saml_issuer_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.saml:addIssuer", settings).await
    }

    pub async fn saml_issuer_delete(&self, issuer: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.saml:deleteIssuer", &json!({"issuer": issuer}))
            .await
    }
}
