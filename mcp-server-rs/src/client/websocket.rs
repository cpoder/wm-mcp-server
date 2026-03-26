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
}
