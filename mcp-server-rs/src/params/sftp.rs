use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SftpServerAliasCreateParam {
    #[schemars(
        description = "JSON string with server alias settings: alias, hostName, port, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SftpServerAliasNameParam {
    #[schemars(description = "SFTP server alias name")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SftpUserAliasCreateParam {
    #[schemars(
        description = "JSON string with user alias settings: alias, userName, authenticationType, password, sftpServerAlias, etc."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SftpUserAliasNameParam {
    #[schemars(description = "SFTP user alias name")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SftpTestConnectionParam {
    #[schemars(description = "SFTP user alias name to test connection for")]
    pub alias: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
