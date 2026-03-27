use serde_json::{Value, json};

impl super::ISClient {
    pub async fn ip_access_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.net:ipRuleList").await
    }

    pub async fn ip_access_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.net:ipRuleAdd", settings).await
    }

    pub async fn ip_access_delete(&self, ip: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.net:ipRuleDelete", &json!({"ip": ip}))
            .await
    }

    pub async fn ip_access_change_type(&self, access_type: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.net:changeIPAccessType",
            &json!({"type": access_type}),
        )
        .await
    }
}
