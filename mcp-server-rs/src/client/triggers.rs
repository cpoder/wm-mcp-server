use serde_json::{Value, json};

impl super::ISClient {
    // ── Pub/Sub Trigger Management ─────────────────────────────

    pub async fn trigger_report(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.triggers:getTriggerReport").await
    }

    pub async fn trigger_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.triggers:createTrigger", settings)
            .await
    }

    pub async fn trigger_delete(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:deleteTrigger",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn trigger_get_properties(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:getProperties",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn trigger_set_properties(
        &self,
        trigger_name: &str,
        properties: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"triggerName": trigger_name});
        if let Some(obj) = properties.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.triggers:setProperties", &payload)
            .await
    }

    pub async fn trigger_suspend(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:suspendTrigger",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn trigger_processing_status(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:getProcessingStatus",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn trigger_retrieval_status(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:getRetrievalStatus",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn trigger_stats(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.triggers:getTriggerStats",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    // ── Messaging Connections ──────────────────────────────────

    pub async fn messaging_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.messaging:getConnectionAliasReport")
            .await
    }

    pub async fn messaging_connection_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.messaging:createConnectionAlias", settings)
            .await
    }

    pub async fn messaging_connection_delete(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.messaging:deleteConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn messaging_connection_enable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.messaging:enableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn messaging_connection_disable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.messaging:disableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn messaging_publishable_doctypes(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.messaging:getPublishableDocumentTypes")
            .await
    }

    pub async fn messaging_csq_count(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.messaging:getCSQMessageCount",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn messaging_csq_clear(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.messaging:clearCSQ",
            &json!({"aliasName": alias_name}),
        )
        .await
    }
}
