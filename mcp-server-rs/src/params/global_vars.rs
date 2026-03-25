use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlobalVarAddParam {
    #[schemars(description = "Variable key name")]
    pub key: String,
    #[schemars(description = "Variable value")]
    pub value: String,
    #[schemars(
        description = "Whether the value is a password/secret (true/false, default: false)"
    )]
    pub is_password: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlobalVarKeyParam {
    #[schemars(description = "Variable key name")]
    pub key: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlobalVarEditParam {
    #[schemars(description = "Variable key name")]
    pub key: String,
    #[schemars(description = "New value")]
    pub value: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
