use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ServiceStatsParam {
    #[schemars(
        description = "Full service name (e.g., \"mypkg.services:myService\"). Omit for all services."
    )]
    pub service_name: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ServerLogParam {
    #[schemars(description = "Number of lines to return from the end of the log")]
    pub num_lines: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
