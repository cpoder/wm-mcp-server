use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAccessParam {
    #[schemars(description = "Port number")]
    pub port: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAccessAddNodesParam {
    #[schemars(description = "Port number")]
    pub port: String,
    #[schemars(
        description = "JSON string with nodes to add: ipAddresses (array), hostNames (array)"
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAccessDeleteNodeParam {
    #[schemars(description = "Port number")]
    pub port: String,
    #[schemars(description = "IP address or hostname to remove")]
    pub node_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAccessSetTypeParam {
    #[schemars(description = "Port number")]
    pub port: String,
    #[schemars(description = "Access type: allow or deny")]
    pub access_type: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
