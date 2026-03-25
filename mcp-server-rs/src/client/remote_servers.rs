use serde_json::{Value, json};

impl super::ISClient {
    pub async fn remote_server_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.remote:serverList").await
    }

    pub async fn remote_server_add(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.remote:addServer", settings)
            .await
    }

    pub async fn remote_server_delete(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.remote:deleteServer", &json!({"alias": alias}))
            .await
    }

    pub async fn remote_server_test(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.remote:testAlias", &json!({"alias": alias}))
            .await
    }
}
