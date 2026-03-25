use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SecuritySettingsUpdateParam {
    #[schemars(
        description = "JSON string with security settings to update (e.g., watt.server.requestCerts, watt.server.requireCerts)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
