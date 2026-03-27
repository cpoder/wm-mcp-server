use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OutboundPasswordStoreParam {
    #[schemars(description = "Password handle identifier")]
    pub handle: String,
    #[schemars(description = "Password value to store")]
    pub password: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OutboundPasswordHandleParam {
    #[schemars(description = "Password handle identifier")]
    pub handle: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
