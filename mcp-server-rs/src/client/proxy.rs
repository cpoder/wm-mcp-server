use serde_json::{Value, json};

impl super::ISClient {
    pub async fn proxy_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.proxy:getProxyServerAliases")
            .await
    }

    pub async fn proxy_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.proxy:createProxyServerAlias", settings)
            .await
    }

    pub async fn proxy_get(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.proxy:getProxyServerDetails",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn proxy_delete(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.proxy:deleteProxyServerAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn proxy_enable(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.proxy:enableProxyServerAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn proxy_disable(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.proxy:disableProxyServerAlias",
            &json!({"alias": alias}),
        )
        .await
    }
}
