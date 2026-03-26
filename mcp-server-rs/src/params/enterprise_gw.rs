use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EgwDosUpdateParam {
    #[schemars(description = "JSON string with DoS protection settings")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
