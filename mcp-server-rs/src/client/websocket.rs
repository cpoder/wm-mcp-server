use serde_json::{Value, json};

impl super::ISClient {
    pub async fn websocket_sessions_by_port(&self, port: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.net.websocket:listSessionsByPort",
            &json!({"port": port}),
        )
        .await
    }

    pub async fn websocket_close_session(&self, session_id: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.net.websocket:closeSession",
            &json!({"sessionId": session_id}),
        )
        .await
    }

    pub async fn websocket_endpoint_create(&self, settings: &Value) -> Result<Value, String> {
        self.invoke_post("wm.server.net.websocket:createWebSocketEndpoint", settings)
            .await
    }

    pub async fn websocket_broadcast(&self, port: &str, message: &str) -> Result<Value, String> {
        self.invoke_post(
            "wm.server.net.websocket:broadcast",
            &json!({"port": port, "message": message}),
        )
        .await
    }
}
