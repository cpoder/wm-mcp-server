use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NsDepNodeParam {
    #[schemars(description = "Fully qualified node name (e.g., \"mypkg.services:myService\")")]
    pub node_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NsDepUnresolvedParam {
    #[schemars(description = "Package name to check for unresolved references")]
    pub package_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NsDepSearchParam {
    #[schemars(description = "Search string (substring match on node names)")]
    pub search_string: String,
    #[schemars(description = "Node type filter: service, record, specification, or empty for all")]
    pub node_type: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NsDepRefactorParam {
    #[schemars(description = "Current fully qualified name of the node to rename/move")]
    pub old_name: String,
    #[schemars(description = "New fully qualified name")]
    pub new_name: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
