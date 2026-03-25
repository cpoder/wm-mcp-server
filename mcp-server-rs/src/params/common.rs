use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InstanceOnlyParam {
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShutdownParam {
    #[schemars(description = "If true, restart the server instead of stopping it")]
    pub bounce: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MapsetValueParam {
    #[schemars(description = "The string value to encode")]
    pub value: String,
}
