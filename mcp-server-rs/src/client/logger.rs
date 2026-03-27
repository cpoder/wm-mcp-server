use serde_json::Value;

impl super::ISClient {
    pub async fn logger_list(&self) -> Result<Value, String> {
        self.admin_get("/admin/logger").await
    }

    pub async fn logger_get(&self, name: &str) -> Result<Value, String> {
        self.admin_get(&format!("/admin/logger/{}", urlencoding::encode(name)))
            .await
    }

    pub async fn logger_update(&self, name: &str, settings: &Value) -> Result<Value, String> {
        self.admin_patch(
            &format!("/admin/logger/{}", urlencoding::encode(name)),
            settings,
        )
        .await
    }

    pub async fn logger_server_config_get(&self) -> Result<Value, String> {
        self.admin_get("/admin/logger/serverconfig").await
    }

    pub async fn logger_server_config_update(&self, settings: &Value) -> Result<Value, String> {
        self.admin_patch("/admin/logger/serverconfig", settings)
            .await
    }
}
