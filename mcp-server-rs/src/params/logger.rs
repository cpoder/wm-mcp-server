use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerNameParam {
    #[schemars(description = "Logger name (e.g., \"Default\", \"Error\", \"Security\")")]
    pub name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerUpdateParam {
    #[schemars(description = "Logger name")]
    pub name: String,
    #[schemars(
        description = "JSON string with logger settings: logLevel (Trace/Debug/Info/Warn/Error/Fatal/Off)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerServerConfigParam {
    #[schemars(description = "JSON string with server logging config settings")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
