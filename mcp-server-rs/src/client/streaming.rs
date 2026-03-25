use serde_json::{Value, json};

impl super::ISClient {
    // ── Streaming (WmStreaming) ────────────────────────────────────────

    pub async fn streaming_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getConnectionAliasReport")
            .await
    }

    pub async fn streaming_connection_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:createConnectionAlias", settings)
            .await
    }

    pub async fn streaming_connection_enable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:enableConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_disable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:disableConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_delete(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:deleteConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_connection_test(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:testConnectionAlias",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_providers(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getAvailableProviders")
            .await
    }

    pub async fn streaming_event_source_list(
        &self,
        alias_name: Option<&str>,
    ) -> Result<Value, String> {
        let mut payload = json!({});
        if let Some(name) = alias_name {
            payload
                .as_object_mut()
                .unwrap()
                .insert("aliasName".into(), json!(name));
        }
        self.invoke_post("wm.server.streaming:getEventSourceReport", &payload)
            .await
    }

    pub async fn streaming_event_source_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:createEventSourceFlat", settings)
            .await
    }

    pub async fn streaming_event_source_delete(
        &self,
        alias_name: &str,
        reference_id: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:deleteEventSource",
            &json!({"aliasName": alias_name, "referenceId": reference_id}),
        )
        .await
    }

    pub async fn streaming_trigger_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.streaming:getTriggerReport")
            .await
    }

    pub async fn streaming_trigger_enable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.streaming:enableTriggers", &json!({"name": name}))
            .await
    }

    pub async fn streaming_trigger_disable(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:disableTriggers",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn streaming_trigger_suspend(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.streaming:suspendTriggers",
            &json!({"name": name}),
        )
        .await
    }
}
