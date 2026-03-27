use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketEndpointCreateParam {
    #[schemars(description = "JSON string with WebSocket endpoint settings")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketSessionParam {
    #[schemars(description = "WebSocket session ID")]
    pub session_id: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketBroadcastParam {
    #[schemars(description = "Port number")]
    pub port: String,
    #[schemars(description = "Message to broadcast to all connected WebSocket clients")]
    pub message: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
