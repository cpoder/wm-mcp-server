use serde_json::{Value, json};

impl super::ISClient {
    pub async fn audit_logger_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.auditing:getAuditLoggers").await
    }

    pub async fn audit_logger_get(&self, logger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.auditing:getAuditLoggerDetails",
            &json!({"loggerName": logger_name}),
        )
        .await
    }

    pub async fn audit_logger_update(
        &self,
        logger_name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"loggerName": logger_name});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.auditing:setAuditLoggerDetails", &payload)
            .await
    }

    pub async fn audit_logger_enable(&self, logger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.auditing:enableLogger",
            &json!({"loggerName": logger_name}),
        )
        .await
    }

    pub async fn audit_logger_disable(&self, logger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.auditing:disableLogger",
            &json!({"loggerName": logger_name}),
        )
        .await
    }
}
