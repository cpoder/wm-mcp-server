use serde_json::{Value, json};

impl super::ISClient {
    pub async fn outbound_password_store(
        &self,
        handle: &str,
        password: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.outboundPasswords:storePassword",
            &json!({"handle": handle, "password": password}),
        )
        .await
    }

    pub async fn outbound_password_retrieve(&self, handle: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.outboundPasswords:retrievePassword",
            &json!({"handle": handle}),
        )
        .await
    }

    pub async fn outbound_password_remove(&self, handle: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.outboundPasswords:removePassword",
            &json!({"handle": handle}),
        )
        .await
    }
}
