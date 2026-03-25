use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SchedulerTaskAddParam {
    #[schemars(
        description = "JSON string with task settings. Required: service (full path), description, type (once/repeat/complex), target (\"$any\" or server name). For once: startDate (MM/dd/yyyy), startTime (HH:mm:ss). For repeat: interval (milliseconds), startDate, startTime. Optional: endDate, endTime, inputs (JSON string of service input params)."
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SchedulerTaskOidParam {
    #[schemars(description = "Task OID (UUID returned from task creation)")]
    pub oid: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SchedulerTaskUpdateParam {
    #[schemars(description = "Task OID")]
    pub oid: String,
    #[schemars(description = "JSON string with settings to update (same fields as addTask)")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
