use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LdapServerSettingsParam {
    #[schemars(
        description = "JSON string with LDAP server settings: serverName, host, port, baseDN, bindDN, bindPassword, useTLS"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LdapServerNameParam {
    #[schemars(description = "LDAP server configuration name")]
    pub server_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
