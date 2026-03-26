use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HealthIndicatorNameParam {
    #[schemars(description = "Health indicator name")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HealthIndicatorUpdateParam {
    #[schemars(description = "Health indicator name")]
    pub name: String,
    #[schemars(
        description = "JSON string with health indicator settings to update: enabled, threshold, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
