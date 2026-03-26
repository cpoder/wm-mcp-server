use serde_json::{Value, json};

impl super::ISClient {
    pub async fn ws_provider_endpoint_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.ws:listAllProviderEndpoints")
            .await
    }

    pub async fn ws_consumer_endpoint_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.ws:listAllConsumerEndpoints")
            .await
    }

    pub async fn ws_wsdl_get(&self, wsd_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.ws:getWsdl",
            &json!({"WSDescriptor_name": wsd_name}),
        )
        .await
    }

    pub async fn rest_resource_list(&self) -> Result<Value, String> {
        self.invoke_get("wm.server.restv2:listAllRESTResources")
            .await
    }

    pub async fn openapi_doc_get(&self, rad_name: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.openapi:getOpenAPIDoc",
            &json!({"radName": rad_name, "openapi.json": "true"}),
        )
        .await
    }

    pub async fn openapi_generate_provider(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.openapi:generateProvider", settings)
            .await
    }

    pub async fn openapi_generate_consumer(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.openapi:generateConsumer", settings)
            .await
    }

    pub async fn openapi_refresh_provider(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.openapi:refreshProvider", settings)
            .await
    }
}
