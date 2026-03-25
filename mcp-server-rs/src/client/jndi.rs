use serde_json::{Value, json};

impl super::ISClient {
    pub async fn jndi_alias_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jndi:getJNDIAliases").await
    }

    pub async fn jndi_alias_get(&self, jndi_alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jndi:getJNDIAliasData",
            &json!({"jndiAliasName": jndi_alias_name}),
        )
        .await
    }

    pub async fn jndi_alias_set(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jndi:setJNDIAliasData", settings)
            .await
    }

    pub async fn jndi_alias_delete(&self, jndi_alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jndi:deleteJNDIAliasData",
            &json!({"jndiAliasName": jndi_alias_name}),
        )
        .await
    }

    pub async fn jndi_test_lookup(
        &self,
        jndi_alias_name: &str,
        lookup_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jndi:testJNDILookup",
            &json!({"jndiAliasName": jndi_alias_name, "lookupName": lookup_name}),
        )
        .await
    }

    pub async fn jndi_templates(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jndi:getJNDIAliasTemplates")
            .await
    }
}
