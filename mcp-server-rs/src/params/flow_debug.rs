use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowDebugStartParam {
    #[schemars(description = "Full service name to debug (e.g., \"mypkg.services:myService\")")]
    pub service: String,
    #[schemars(description = "JSON string of initial pipeline inputs (optional)")]
    pub pipeline: Option<String>,
    #[schemars(description = "Stop at first step (default: true)")]
    pub stop_at_start: Option<bool>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowDebugCommandParam {
    #[schemars(description = "Debug session OID (from flow_debug_start)")]
    pub debug_oid: String,
    #[schemars(description = "Debug command: stepOver, stepIn, stepOut, resume, stop")]
    pub command: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowDebugOidParam {
    #[schemars(description = "Debug session OID")]
    pub debug_oid: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowDebugBreakpointParam {
    #[schemars(description = "Debug session OID")]
    pub debug_oid: String,
    #[schemars(
        description = "JSON string with breakpoints to insert: {\"breakPoint1\": {\"serviceName\": \"...\", \"path\": \"/0\"}, ...}"
    )]
    pub breakpoints: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlowDebugSetPipelineParam {
    #[schemars(description = "Debug session OID")]
    pub debug_oid: String,
    #[schemars(description = "JSON string with pipeline values to set")]
    pub pipeline: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
