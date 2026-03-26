use serde_json::{Value, json};

impl super::ISClient {
    pub async fn sftp_server_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.sftpclient:listServerAliases")
            .await
    }

    pub async fn sftp_server_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.sftpclient:createServerAlias", settings)
            .await
    }

    pub async fn sftp_server_get(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.sftpclient:getServerAliasInfo",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn sftp_server_delete(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.sftpclient:deleteServerAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn sftp_user_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.sftpclient:listUserAliases")
            .await
    }

    pub async fn sftp_user_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.sftpclient:createUserAlias", settings)
            .await
    }

    pub async fn sftp_user_get(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.sftpclient:getUserAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn sftp_user_delete(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.sftpclient:removeUserAlias",
            &json!({"alias": alias}),
        )
        .await
    }

    pub async fn sftp_test_connection(&self, alias: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.sftpclient:testConnection",
            &json!({"alias": alias}),
        )
        .await
    }
}
