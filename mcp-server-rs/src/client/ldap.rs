use serde_json::{Value, json};

impl super::ISClient {
    pub async fn ldap_settings_get(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.ldap:getSettings").await
    }

    pub async fn ldap_server_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.ldap:addConfiguredServer", settings)
            .await
    }

    pub async fn ldap_server_edit(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.ldap:editConfiguredServer", settings)
            .await
    }

    pub async fn ldap_server_delete(&self, server_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ldap:deleteConfiguredServer",
            &json!({"serverName": server_name}),
        )
        .await
    }
}
