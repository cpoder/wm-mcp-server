use serde_json::{Value, json};

impl super::ISClient {
    pub async fn egw_rules_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.enterprisegateway:getRulesList")
            .await
    }

    pub async fn egw_dos_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.enterprisegateway:getDOS").await
    }

    pub async fn egw_dos_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.enterprisegateway:saveDOS", settings)
            .await
    }

    pub async fn egw_denied_ip_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.enterprisegateway:getDeniedIPList")
            .await
    }

    pub async fn egw_rule_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.enterprisegateway:addRule", settings)
            .await
    }

    pub async fn egw_rule_delete(&self, rule_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.enterprisegateway:deleteRule",
            &json!({"ruleName": rule_name}),
        )
        .await
    }

    pub async fn egw_rule_update(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.enterprisegateway:updateRule", settings)
            .await
    }
}
