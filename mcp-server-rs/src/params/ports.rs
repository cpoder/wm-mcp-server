use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortKeyPkgParam {
    #[schemars(description = "Listener key from port_list (e.g., \"HTTPListener@5555\")")]
    pub port_key: String,
    #[schemars(description = "Package that owns the listener (e.g., \"WmRoot\")")]
    pub pkg: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortAddParam {
    #[schemars(
        description = "JSON string with listener configuration including \"factoryKey\" and \"pkg\""
    )]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PortUpdateParam {
    #[schemars(description = "Listener key (e.g., \"HTTPListener@5555\")")]
    pub port_key: String,
    #[schemars(description = "Package that owns the listener (e.g., \"WmRoot\")")]
    pub pkg: String,
    #[schemars(description = "JSON string with properties to update")]
    pub settings: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
