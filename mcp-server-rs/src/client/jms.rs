use serde_json::{Value, json};

impl super::ISClient {
    pub async fn jms_connection_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jms:getConnectionAliasReport")
            .await
    }

    pub async fn jms_connection_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jms:createConnectionAlias", settings)
            .await
    }

    pub async fn jms_connection_update(
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
        self.invoke_post("wm.server.jms:updateConnectionAlias", &payload)
            .await
    }

    pub async fn jms_connection_delete(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:deleteConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn jms_connection_enable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:enableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn jms_connection_disable(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:disableConnectionAlias",
            &json!({"aliasName": alias_name}),
        )
        .await
    }

    pub async fn jms_trigger_report(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.jms:getTriggerReport").await
    }

    pub async fn jms_trigger_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.jms:createJMSTrigger", settings)
            .await
    }

    pub async fn jms_trigger_update(
        &self,
        trigger_name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"triggerName": trigger_name});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.jms:updateJMSTrigger", &payload)
            .await
    }

    pub async fn jms_trigger_delete(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:deleteJMSTrigger",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn jms_trigger_enable(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:enableJMSTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn jms_trigger_disable(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:disableJMSTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn jms_trigger_suspend(&self, trigger_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:suspendJMSTriggers",
            &json!({"triggerName": trigger_name}),
        )
        .await
    }

    pub async fn jms_destination_list(&self, alias_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.jms:listDestinations",
            &json!({"aliasName": alias_name}),
        )
        .await
    }
}
