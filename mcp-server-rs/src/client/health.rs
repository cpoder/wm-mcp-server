use serde_json::{Value, json};

impl super::ISClient {
    pub async fn health_indicators_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.healthindicators:getAllHealthIndicators")
            .await
    }

    pub async fn health_indicator_get(&self, name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.healthindicators:getHealthIndicator",
            &json!({"name": name}),
        )
        .await
    }

    pub async fn health_indicator_change(
        &self,
        name: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"name": name});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        self.invoke_post("wm.server.healthindicators:changeHealthIndicator", &payload)
            .await
    }
}
