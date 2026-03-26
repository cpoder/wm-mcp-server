use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IpRuleAddParam {
    #[schemars(description = "JSON string with IP access rule settings: ip, type")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IpRuleParam {
    #[schemars(description = "IP address for the access rule")]
    pub ip: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
