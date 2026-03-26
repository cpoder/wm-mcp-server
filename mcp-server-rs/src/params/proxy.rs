use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyCreateParam {
    #[schemars(
        description = "JSON string with proxy server alias settings: alias, host, port, protocol, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyAliasNameParam {
    #[schemars(description = "Proxy server alias name")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
