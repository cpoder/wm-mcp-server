use serde_json::{Value, json};

impl super::ISClient {
    pub async fn port_access_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.portAccess:portList").await
    }

    pub async fn port_access_get(&self, port: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.portAccess:getPort", &json!({"port": port}))
            .await
    }

    pub async fn port_access_add_nodes(
        &self,
        port: &str,
        settings: &Value,
    ) -> Result<Value, String> {
        let mut payload = json!({"port": port});
        if let Some(obj) = settings.as_object() {
            for (k, v) in obj {
                payload[k] = v.clone();
            }
        }
        self.invoke_post("wm.server.portAccess:addNodes", &payload)
            .await
    }

    pub async fn port_access_delete_node(
        &self,
        port: &str,
        node_name: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.portAccess:deleteNode",
            &json!({"port": port, "nodeName": node_name}),
        )
        .await
    }

    pub async fn port_access_set_type(
        &self,
        port: &str,
        access_type: &str,
    ) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.portAccess:setType",
            &json!({"port": port, "type": access_type}),
        )
        .await
    }

    pub async fn port_access_reset(&self, port: &str) -> Result<Value, String> {
        self.invoke_post("wm.server.portAccess:resetPort", &json!({"port": port}))
            .await
    }
}
