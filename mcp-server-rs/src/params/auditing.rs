use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditLoggerNameParam {
    #[schemars(
        description = "Logger name (e.g., \"Error Logger\", \"Document Logger\", \"Security Logger\")"
    )]
    pub logger_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditLoggerUpdateParam {
    #[schemars(description = "Logger name")]
    pub logger_name: String,
    #[schemars(
        description = "JSON string with settings to update (isEnabled, isDatabase, isAsynchronous, isGuaranteed, etc.)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
