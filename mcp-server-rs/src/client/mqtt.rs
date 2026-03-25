use serde_json::{Value, json};

impl super::ISClient {
    pub async fn mqtt_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.mqtt:getConnectionAliasReport")
            .await
    }

    pub async fn mqtt_connection_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.mqtt:createConnectionAlias", settings)
            .await
    }

    pub async fn mqtt_connection_update(
        &self,
        alias_name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"aliasName": alias_name});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.mqtt:updateConnectionAlias", &payload)
            .await
    }

    pub async fn mqtt_connection_delete(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:deleteConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn mqtt_connection_enable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:enableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn mqtt_connection_disable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:disableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn mqtt_trigger_report(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.mqtt:getTriggerReport").await
    }

    pub async fn mqtt_trigger_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.mqtt:createTrigger", settings)
            .await
    }

    pub async fn mqtt_trigger_delete(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:deleteTrigger",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn mqtt_trigger_enable(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:enableTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn mqtt_trigger_disable(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:disableTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn mqtt_trigger_suspend(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.mqtt:suspendTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }
}
