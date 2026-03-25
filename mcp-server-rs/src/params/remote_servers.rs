use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoteServerAddParam {
    #[schemars(
        description = "JSON string with server settings: alias (name), host, port, user, pass. Optional: ssl (yes/no), acl, keepalive, timeout."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoteServerAliasParam {
    #[schemars(description = "Remote server alias name")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
