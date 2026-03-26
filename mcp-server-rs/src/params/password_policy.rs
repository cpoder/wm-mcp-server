use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PasswordPolicyUpdateParam {
    #[schemars(description = "JSON string with password expiry settings to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
